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
    lodash: _,
    config
} = require("./../sdk");

const assert = require("assert");
const log = console.log;

const BUCKET_CONTRACT_NAME = config.DDC_BUCKET_CONTRACT_NAME;
const SEED = config.ACTOR_SEED;
const RPC = config.DEVNET_RPC_ENDPOINT;

deploymentRegistry.initContracts();

async function main() {
    const {api, chainName, getExplorerUrl} = await connect(RPC);
    log("Connected to blockchain:", chainName);

    const account = accountFromUri(SEED);
    log("From account", account.address);

    const bucketContract = getContract(BUCKET_CONTRACT_NAME, chainName, api);
    log("Using bucket contract", BUCKET_CONTRACT_NAME, "at", bucketContract.address.toString());

    const txOptions = {
        value: 0n,
        gasLimit: 200_000n * MGAS,
    };

    const txOptionsPay = {
        value: 10n * CERE,
        gasLimit: 200_000n * MGAS,
    };

    // Test data.
    const managerId = account.address;
    const anyAccountId = account.address;
    const rentPerMonth = 10n * CERE;
    const nodeParams = "{\"url\":\"https://ddc-123.cere.network/bucket/{BUCKET_ID}\"}";
    const capacity = 1e6;
    const vNodes = [1,2,3,4,5,6];
    const clusterResource = 10;
    const bucketResource = 5;
    const bucketParams = "{}";
    const nodePubKey = randomAccount().address;

    let nodeId;
    {
        log("Setup a node…");
        const tx = bucketContract.tx
            .nodeCreate(txOptionsPay, rentPerMonth, nodeParams, capacity, 'ADDING', nodePubKey);

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        nodeId = ddcBucket.findCreatedNodeId(events);
        log("New NodeId", nodeId, "\n");
    }
    
    {
        log("Trust the cluster manager…");
        const tx = bucketContract.tx
            .nodeTrustManager(txOptionsPay, managerId);

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        printEvents(result);
        log();
    }

    let clusterId;
    {
        log("Setup a cluster…");
        let clusterParams = "{}";
        const tx = bucketContract.tx
            .clusterCreate(txOptionsPay, managerId, vNodes, [nodeId], clusterParams);
        
        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        clusterId = ddcBucket.findCreatedClusterId(events);
        log("New ClusterId", clusterId, "\n");
    }

    {
        log("Reserve some resources for the cluster…");
        const tx = bucketContract.tx
            .clusterReserveResource(txOptions, clusterId, clusterResource);

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        printEvents(result);
        log();
    }

    {
        log("Changing node tag…");
        const tx = bucketContract.tx
            .clusterChangeNodeTag(txOptions, nodeId, 'ACTIVE');

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        printEvents(result);
        log();
    }

    let bucketId;
    {
        log("Create a bucket…");
        const tx = bucketContract.tx
            .bucketCreate(txOptionsPay, bucketParams, clusterId, null);

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        bucketId = ddcBucket.findCreatedBucketId(events);
        log("New BucketId", bucketId, "\n");
    }

    {
        log("Topup the account…");
        const tx = bucketContract.tx
            .accountDeposit(txOptionsPay);

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        printEvents(result);
        log();
    }

    {
        log("Allocate some resources for the bucket…");
        const tx = bucketContract.tx
            .bucketAllocIntoCluster(txOptions, bucketId, bucketResource);

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        printEvents(result);
        log();
    }

    {
        log("Collect payment from Bucket to Cluster…");
        const tx = bucketContract.tx
            .bucketSettlePayment(txOptions, bucketId);

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        printEvents(result);
        log();
    }

    {
        log("Distribute payment from Cluster to Providers…");
        const tx = bucketContract.tx
            .clusterDistributeRevenues(txOptions, clusterId);

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        printEvents(result);
        log();
    }

    log("    ----    \n");

    {
        log("Read the node status…");
        const {result, output} = await bucketContract.query
            .nodeGet(anyAccountId, txOptions, nodeId);

        if (!result.isOk) assert.fail(result.asErr);
        log("Node", output.toHuman(), "\n");
    }

    {
        log("Read the cluster status…");
        const {result, output} = await bucketContract.query
            .clusterGet(anyAccountId, txOptions, clusterId);

        if (!result.isOk) assert.fail(result.asErr);
        log("Cluster", output.toHuman(), "\n");
    }

    {
        log("Read the bucket status…");
        let {result, output} = await bucketContract.query
            .bucketGet(anyAccountId, txOptions, bucketId);

        if (!result.isOk) assert.fail(result.asErr);
        log("Bucket", output.toHuman(), "\n");
    }

    process.exit(0);
}

function printEvents(result) {
    const events = result.contractEvents || [];
    //log("EVENTS", JSON.stringify(events, null, 4));
    log(events.length, "events");
    return events;
}

function printGas(result) {
    let gas = _.get(result, "dispatchInfo.weight", 0);
    log(parseInt(gas / 1e6), "MGas");
}

main().then(log, log);
