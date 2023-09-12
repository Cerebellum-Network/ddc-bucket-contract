const fs = require("fs");
const {
  connect,
  accountFromUri,
  sendTx,
  getContract,
  CERE,
  MGAS,
  ddcBucket,
  deploymentRegistry,
  config
} = require("../sdk/src");
const ddcConfig = require('./ddcConfig.js');
const log = console.log;

const ENV = process.env.ENV;
const ddcEnvConfig = ddcConfig[ENV];
const SUPERADMIN_MNEMONIC = process.env.SUPERADMIN;
const DDC_CONTRACT_ADDRESS = process.env.DDC_CONTRACT;

if (!ddcEnvConfig) {
    console.error("Please provide ENV as one of ", Object.keys(ddcConfig));
    process.exit(-1);
}
console.log(ddcEnvConfig);

if (!SUPERADMIN_MNEMONIC) {
    console.error("Please provide SUPERADMIN seed");
    process.exit(-1);
}

if (!DDC_CONTRACT_ADDRESS) {
    console.error("Please provide DDC_CONTRACT address");
    process.exit(-1);
}

deploymentRegistry.initContract(config.DDC_BUCKET_CONTRACT_NAME, ENV, DDC_CONTRACT_ADDRESS);

async function main() {
    const {api, chainName, getExplorerUrl} = await connect(ddcEnvConfig.blockchainUrl);
    log("Connected to blockchain:", chainName);

    const sadmin = accountFromUri(SUPERADMIN_MNEMONIC);
    console.log(`Superadmin: ${sadmin.address}`);

    const bucketContract = getContract(config.DDC_BUCKET_CONTRACT_NAME, ENV, api);
    log("Using bucket contract", config.DDC_BUCKET_CONTRACT_NAME, "at", bucketContract.address.toString());

    const txOptions = {
      storageDepositLimit: null,
      gasLimit: 100_000_000_000n,
    };
    
    console.log( "\n", `Setup Started`, "\n");

    {
        log(`Setting USD per CERE rate ...`);
        const tx = bucketContract.tx.accountSetUsdPerCere(
            txOptions, 
            1000n * CERE
        );
        const result = await sendTx(sadmin, tx);
        log(getExplorerUrl(result), "\n");
    }

    {
        log(`Granting trusted managers permissions ...`);
        const tx = bucketContract.tx.grantTrustedManagerPermission(
            txOptions, 
            sadmin.address
        );
        const result = await sendTx(sadmin, tx);
        log(getExplorerUrl(result), "\n");
    }

    for (let i = 0; i < ddcEnvConfig.clusters.length; i++) {
        const cluster = ddcEnvConfig.clusters[i];
        const clusterParams = JSON.stringify(cluster.params);
        const resourcePerVNode = 100000n;

        console.log(`Creating Cluster ${i} ...`);
        const clusterCreateTx = bucketContract.tx.clusterCreate(
            txOptions,
            clusterParams,
            resourcePerVNode
        );
        const result = await sendTx(sadmin, clusterCreateTx);
        log(getExplorerUrl(result), "\n");
        let { clusterId } = ddcBucket.findClusterCreatedEvent(result.contractEvents || []);
        console.log(`Cluster ${clusterId} created`);

        for (let j = 0; j < cluster.storageNodes.length; j++) {
            const storageNode = cluster.storageNodes[j];
            const storageNodeKey = storageNode.pubKey;
            const storageNodeParams = JSON.stringify(storageNode.params);
            const vNodes = storageNode.vNodes;
            const storageNodeCapacity = 100000n;
            const rentVNodePerMonth = 1n * CERE;

            console.log(`Creating Storage node ${storageNodeKey} ...`);
            const nodeCreateTx = bucketContract.tx.nodeCreate(
                txOptions, 
                storageNodeKey,
                storageNodeParams,
                storageNodeCapacity,
                rentVNodePerMonth,
            );
            const result1 = await sendTx(sadmin, nodeCreateTx);
            log(getExplorerUrl(result1), "\n");

            console.log(`Adding Storage node ${storageNodeKey} to Cluster ${clusterId} ...`);
            const clusterAddNodeTx = bucketContract.tx.clusterAddNode(
                txOptions,
                clusterId,
                storageNodeKey,
                vNodes
            )
            const result2 = await sendTx(sadmin, clusterAddNodeTx);
            log(getExplorerUrl(result2), "\n");

            const storageNodeStatus = 'ACTIVE';
            console.log(`Setting '${storageNodeStatus}' status ${storageNodeKey} in Cluster ${clusterId} ...`);
            const clusterSetNodeStatusTx = bucketContract.tx.clusterSetNodeStatus(
                txOptions,
                clusterId,
                storageNodeKey,
                storageNodeStatus,
            )
            const result3 = await sendTx(sadmin, clusterSetNodeStatusTx);
            log(getExplorerUrl(result3), "\n");
        }

        for (let j = 0; j < cluster.cdnNodes.length; j++) {
            const cdnNode = cluster.cdnNodes[j];
            const cdnNodeKey = cdnNode.pubKey;
            const cdnNodeParams = JSON.stringify(cdnNode.params);
            
            console.log(`Creating CDN node ${cdnNodeKey} ...`);
            const cdnNodeCreateTx = bucketContract.tx.cdnNodeCreate(
                txOptions, 
                cdnNodeKey,
                cdnNodeParams
            );
            const result1 = await sendTx(sadmin, cdnNodeCreateTx);
            log(getExplorerUrl(result1), "\n");

            console.log(`Adding CDN node ${cdnNodeKey} to Cluster ${clusterId} ...`);
            const clusterAddCdnNodeTx = bucketContract.tx.clusterAddCdnNode(
                txOptions,
                clusterId,
                cdnNodeKey,
            )
            const result2 = await sendTx(sadmin, clusterAddCdnNodeTx);
            log(getExplorerUrl(result2), "\n");

            const cdnNodeStatus = 'ACTIVE';
            console.log(`Setting '${cdnNodeStatus}' status ${cdnNodeKey} in Cluster ${clusterId} ...`);
            const clusterSetCdnNodeStatusTx = bucketContract.tx.clusterSetCdnNodeStatus(
                txOptions,
                clusterId,
                cdnNodeKey,
                cdnNodeStatus,
            )
            const result3 = await sendTx(sadmin, clusterSetCdnNodeStatusTx);
            log(getExplorerUrl(result3), "\n");
        }
    }

    console.log( "\n", `Setup Finished`, "\n");
    process.exit(0);
}


main().then(log, log);