const {
    connect,
    accountFromUri,
    registerABI,
    getCodeDeployer,
    sendTx,
    registerContract,
    getBlueprint,
    getContract,
    CERE,
    MGAS,
    ddcBucket,
} = require("./sdk");

const assert = require("assert");
const log = console.log;

const CONTRACT_NAME = "ddc_bucket";
const SEED = "//Alice";
const RPC = "wss://rpc.testnet.cere.network:9945";

const ENDOWMENT = 10n * CERE;


async function main() {
    const {api, chainName, getExplorerUrl} = await connect(RPC);
    log("Connected to blockchain:", chainName);

    const account = accountFromUri(SEED);
    log("From account", account.address);

    const contract = getContract(CONTRACT_NAME, chainName, api);
    log("Using contract", CONTRACT_NAME, "at", contract.address.toString());

    const txOptions = {
        value: 0n,
        gasLimit: -1, //100_000n * MGAS,
    };
    const txOptionsPay = {
        value: 10n * CERE,
        gasLimit: -1, //100_000n * MGAS,
    };

    // Test data.
    const managerId = account.address;
    const anyAccountId = account.address;
    const rent_per_month = 10n * CERE;
    const node_params = "{\"url\":\"https://ddc-123.cere.network/bucket/{BUCKET_ID}\"}";
    const capacity = 100;
    const cluster_resource = 10;
    const bucket_resource = 5;
    const bucket_params = "{}";

    let nodeId;
    {
        log("Setup a node…");
        const tx = contract.tx
            .nodeCreate(txOptionsPay, rent_per_month, node_params, capacity);

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
        nodeId = ddcBucket.findCreatedNodeId(events);
        log("New NodeId", nodeId, "\n");
    }

    let clusterId;
    {
        log("Setup a cluster…");
        let cluster_params = "{}";
        const tx = contract.tx
            .clusterCreate(txOptionsPay, managerId, 6, [nodeId], cluster_params);

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
        clusterId = ddcBucket.findCreatedClusterId(events);
        log("New ClusterId", clusterId, "\n");
    }
    {
        log("Reserve some resources for the cluster…");
        const tx = contract.tx
            .clusterReserveResource(txOptionsPay, clusterId, cluster_resource);

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
        log();
    }

    let bucketId;
    {
        log("Create a bucket…");
        const tx = contract.tx
            .bucketCreate(txOptionsPay, bucket_params, clusterId);

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
        bucketId = ddcBucket.findCreatedBucketId(events);
        log("New BucketId", bucketId, "\n");
    }
    {
        log("Topup the account…");
        const tx = contract.tx
            .accountDeposit(txOptionsPay);

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
        log();
    }
    {
        log("Allocate some resources for the bucket…");
        const tx = contract.tx
            .bucketAllocIntoCluster(txOptionsPay, bucketId, bucket_resource);

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
        log();
    }
    log("    ----    \n");

    {
        log("Read the node status…");
        const {result, output} = await contract.query
            .nodeGet(anyAccountId, txOptions, nodeId);

        if (!result.isOk) assert.fail(result.asErr);
        log("Node", output.toHuman(), "\n");
    }
    {
        log("Read the cluster status…");
        const {result, output} = await contract.query
            .clusterGet(anyAccountId, txOptions, clusterId);

        if (!result.isOk) assert.fail(result.asErr);
        log("Cluster", output.toHuman(), "\n");
    }
    {
        log("Read the bucket status…");
        let {result, output} = await contract.query
            .bucketGet(anyAccountId, txOptions, bucketId);

        if (!result.isOk) assert.fail(result.asErr);
        log("Bucket", output.toHuman(), "\n");
    }

    process.exit(0);
}

main().then(log, log);
