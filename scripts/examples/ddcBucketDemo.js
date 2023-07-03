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
const RPC = process.env.ENV == 'devnet' 
    ? config.DEVNET_RPC_ENDPOINT 
    : process.env.ENV == 'testnet' 
        ? config.TESTNET_RPC_ENDPOINT 
        : config.LOCAL_RPC_ENDPOINT;

deploymentRegistry.initDefaultContracts();

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
    const clusterManagerId = account.address;
    const nodePubKey = randomAccount().address;
    const cdnNodePubKey = randomAccount().address;

    const anyAccountId = account.address;

    const bucketResource = 5;
    const bucketParams = "{}";

    let nodeKey;
    {
        log("Create a Storage node...");
        const nodeParams = "{\"url\":\"https://ddc-123.cere.network/storage/0\"}";
        const capacity = 1e6;
        const rentPerMonth = 10n * CERE;
        const tx = bucketContract.tx.nodeCreate(
            txOptionsPay, 
            nodePubKey,
            nodeParams,
            capacity,
            rentPerMonth,
        );

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        let event = ddcBucket.findNodeCreatedEvent(events);
        nodeKey = event.nodeKey;
        log("NodeCreated", event, "\n");
    }

    let cdnNodeKey;
    {
        log("Create a CDN node...");
        const cdnNodeParams = "{\"url\":\"https://ddc-123.cere.network/cdn/0\"}";
        const tx = bucketContract.tx.cdnNodeCreate(
            txOptionsPay, 
            cdnNodePubKey, 
            cdnNodeParams
        );

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        let event = ddcBucket.findCdnNodeCreatedEvent(events);
        cdnNodeKey = event.cdnNodeKey;
        log("CdnNodeCreated", event, "\n");
    }

    let clusterId;
    {
        log("Setup a cluster...");
        let clusterParams = "{}";
        const resourcePerVNode = 10;
        const tx = bucketContract.tx.clusterCreate(
            txOptionsPay,
            clusterParams,
            resourcePerVNode
        );
        
        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        let event = ddcBucket.findClusterCreatedEvent(events);
        clusterId = event.clusterId;
        log("ClusterCreated", event, "\n");
    }

    let trustedManagerId;
    {
        log("Trust the cluster manager...");
        const tx = bucketContract.tx.grantTrustedManagerPermission(
            txOptionsPay, 
            clusterManagerId
        );
        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        let event = ddcBucket.findPermissionGrantedEvent(events);
        trustedManagerId = event.accountId;
        log("PermissionGranted", event, "\n");
    }

    {
        log("Adding Storage node to the cluster...");
        const vNodes = [1,2,3,4,5,6];
        const tx = bucketContract.tx.clusterAddNode(
            txOptionsPay, 
            clusterId,
            nodeKey,
            vNodes
        );

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        let event = ddcBucket.findClusterNodeAddedEvent(events);
        log("ClusterNodeAdded", event, "\n");
    }

    {
        log("Adding CDN node to the cluster...");
        const tx = bucketContract.tx.clusterAddCdnNode(
            txOptionsPay, 
            clusterId,
            cdnNodeKey,
        );

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        let event = ddcBucket.findClusterCdnNodeAddedEvent(events);
        cdnNodeKey = event.cdnNodeKey;
        log("ClusterCdnNodeAdded", event, "\n");
    }

    {
        log("Changing Storage node status...");
        const tx = bucketContract.tx.clusterSetNodeStatus(
            txOptions, 
            clusterId,
            nodeKey,
            'ACTIVE'
        );

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        let event = ddcBucket.findClusterNodeStatusSetEvent(events);
        log("ClusterNodeStatusSet", event, "\n");
    }

    {
        log("Changing CDN node status...");
        const tx = bucketContract.tx.clusterSetCdnNodeStatus(
            txOptions, 
            clusterId,
            cdnNodeKey,
            'ACTIVE'
        );

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        let event = ddcBucket.findClusterCdnNodeStatusSetEvent(events);
        log("ClusterCdnNodeStatusSet", event, "\n");
    }

    let bucketId;
    {
        log("Create a bucket...");
        const tx = bucketContract.tx.bucketCreate(
            txOptionsPay, 
            bucketParams, 
            clusterId, 
            null
        );

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        let event = ddcBucket.findBucketCreatedEvent(events);
        bucketId = event.bucketId;
        log("BucketCreated", event, "\n");
    }

    {
        log("Topup the account...");
        const tx = bucketContract.tx.accountDeposit(
            txOptionsPay
        );

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        let event = ddcBucket.findDepositEvent(events);
        log("Deposit", event, "\n");
    }

    {
        log("Allocate some resources for the bucket...");
        const tx = bucketContract.tx.bucketAllocIntoCluster(
            txOptions, 
            bucketId, 
            bucketResource
        );

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        let event = ddcBucket.findBucketAllocatedEvent(events);
        log("BucketAllocated", event, "\n");
    }

    {
        log("Collect payment from Bucket to Cluster...");
        const tx = bucketContract.tx.bucketSettlePayment(
            txOptions, 
            bucketId
        );

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        let event = ddcBucket.findBucketSettlePaymentEvent(events)
        log("BucketSettlePayment", event, "\n");
    }

    {
        log("Distribute payment from Cluster to Providers...");
        const tx = bucketContract.tx.clusterDistributeRevenues(
            txOptions, 
            clusterId
        );

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);
        let event = ddcBucket.findClusterDistributeRevenuesEvent(events);
        log("ClusterDistributeRevenues", event, "\n");
    }

    log("    ----    \n");

    {
        log("Read the node status…");
        const {result, output} = await bucketContract.query
            .nodeGet(anyAccountId, txOptions, nodeKey);

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
