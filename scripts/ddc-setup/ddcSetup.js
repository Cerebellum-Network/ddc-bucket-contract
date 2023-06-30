const fs = require("fs");
const {
  connect,
  accountFromUri,
  sendTx,
  getContract,
  CERE,
  MGAS,
  ddcBucket,
  randomAccount,
  deploymentRegistry,
  config
} = require("../sdk/src");
const ddcConfig = require('./ddcConfig.js');
const log = console.log;

const DDC_BUCKET_CONTRACT_NAME = config.DDC_BUCKET_CONTRACT_NAME;
const INIT_ENV = process.env.INIT_ENV;
const SUPERADMIN_MNEMONIC = process.env.SUPERADMIN;


const ddcEnvConfig = ddcConfig[INIT_ENV];
if (ddcEnvConfig === undefined) {
  console.error("Please provide INIT_ENV as one of ", Object.keys(ddcConfig));
  process.exit(-1);
}
console.log(ddcEnvConfig);

deploymentRegistry.initContract(
  DDC_BUCKET_CONTRACT_NAME, 
  INIT_ENV, 
  ddcEnvConfig.contract_address
);


async function main() {
    const {api, chainName, getExplorerUrl} = await connect(ddcEnvConfig.ws_provider);
    log("Connected to blockchain:", chainName);

    const sadmin = accountFromUri(SUPERADMIN_MNEMONIC);
    console.log(`Superadmin: ${sadmin.address}`);

    const bucketContract = getContract(DDC_BUCKET_CONTRACT_NAME, INIT_ENV, api);
    log("Using bucket contract", DDC_BUCKET_CONTRACT_NAME, "at", bucketContract.address.toString());

    const txOptions = {
      storageDepositLimit: null,
      gasLimit: 100_000_000_000n,
    };
    
    {
        log("1. accountSetUsdPerCere");
        const tx = bucketContract.tx.accountSetUsdPerCere(
            txOptions, 
            1000n * CERE
        );

        const result = await sendTx(sadmin, tx);
        log(getExplorerUrl(result), "\n");
    }

    {
        log("2. grantTrustedManagerPermission");
        const tx = bucketContract.tx.grantTrustedManagerPermission(
            txOptions, 
            sadmin.address
        );
        const result = await sendTx(sadmin, tx);
        log(getExplorerUrl(result), "\n");
    }

    const cdnNodesKeys = []
    {
        console.log("3. cdnNodeCreate");
        for (let i = 0; i < ddcEnvConfig.cdn_node_params.length; i++) {
            const cdnNodeKey = ddcEnvConfig.cdn_node_params[i].publicKey;
            cdnNodesKeys.push(cdnNodeKey);

            const tx = bucketContract.tx.cdnNodeCreate(
                txOptions,
                cdnNodeKey, 
                JSON.stringify(ddcEnvConfig.cdn_node_params[i])
            );

            const result = await sendTx(sadmin, tx);
            log(getExplorerUrl(result), "\n");
        }
    }

    const storageNodesKeys = []
    {
        console.log("4. nodeCreate");
        for (let i = 0; i < ddcEnvConfig.storage_node_params.length; i++) {
            const param = JSON.stringify(ddcEnvConfig.storage_node_params[i]);
            const user = randomAccount();

            fs.appendFileSync('secrets.txt', `${user.address}: ${user.mnemonic} -- ${INIT_ENV} storage ${i}\n`);
            console.log(`  node ${i}: address ${user.address}, param ${param}`);

            const storageNodeKey = user.address;
            storageNodesKeys.push(storageNodeKey);

            const tx = bucketContract.tx.nodeCreate(
                txOptions, 
                storageNodeKey,
                param,
                100000n,
                1n * CERE
            );

            const result = await sendTx(sadmin, tx);
            log(getExplorerUrl(result), "\n");
        }
    }

    const clustersIds = [];
    {
        for (let key in ddcEnvConfig.cluster) {
            console.log("5. clusterCreate ");

            const tx1 = bucketContract.tx.clusterCreate(
                txOptions,
                JSON.stringify(ddcEnvConfig.cluster[key].param)
            );
            
            const result1 = await sendTx(sadmin, tx1);
            log(getExplorerUrl(result1), "\n");
            const { clusterId } = ddcBucket.findClusterCreatedEvent(result1.contractEvents);
            clustersIds.push(clusterId);

            const tx2 = bucketContract.tx.clusterReserveResource(
                txOptions, 
                clusterId, 
                100000n
            );
            const result2 = await sendTx(sadmin, tx2);
            log(getExplorerUrl(result2), "\n");
        }
    }

    // TODO: Add Storage nodes and CDN nodes to clusters 

    process.exit(0);
}


main().then(log, log);