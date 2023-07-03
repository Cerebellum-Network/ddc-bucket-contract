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
const ENV = process.env.ENV;
const SUPERADMIN_MNEMONIC = process.env.SUPERADMIN;


const ddcEnvConfig = ddcConfig[ENV];
if (!ddcEnvConfig) {
  console.error("Please provide ENV as one of ", Object.keys(ddcConfig));
  process.exit(-1);
}
console.log(ddcEnvConfig);

if (!SUPERADMIN_MNEMONIC) {
    console.error("Please provide SUPERADMIN seed");
    process.exit(-1);
}

deploymentRegistry.initContract(
  DDC_BUCKET_CONTRACT_NAME, 
  ENV, 
  ddcEnvConfig.contract_address
);


async function main() {
    const {api, chainName, getExplorerUrl} = await connect(ddcEnvConfig.ws_provider);
    log("Connected to blockchain:", chainName);

    const sadmin = accountFromUri(SUPERADMIN_MNEMONIC);
    console.log(`Superadmin: ${sadmin.address}`);

    const bucketContract = getContract(DDC_BUCKET_CONTRACT_NAME, ENV, api);
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

            fs.appendFileSync('secrets.txt', `${user.address}: ${user.mnemonic} -- ${ENV} storage ${i}\n`);
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
                JSON.stringify(ddcEnvConfig.cluster[key].param),
                100000n
            );
            
            const result1 = await sendTx(sadmin, tx1);
            log(getExplorerUrl(result1), "\n");
            let { clusterId } = ddcBucket.findClusterCreatedEvent(result1.contractEvents || []);
            clustersIds.push(clusterId);
        }
    }

    // TODO: Add Storage nodes and CDN nodes to clusters 

    process.exit(0);
}


main().then(log, log);