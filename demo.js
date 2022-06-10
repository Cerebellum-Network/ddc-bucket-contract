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
    ddcNftRegistry,
    lodash: _,
} = require("./sdk");

const assert = require("assert");
const log = console.log;

const BUCKET_CONTRACT_NAME = "ddc_bucket";
const NFT_REGISTRY_CONTRACT_NAME = "ddc_nft_registry";

const SEED = "//Alice";
const RPC = "wss://rpc.testnet.cere.network:9945";

const ENDOWMENT = 10n * CERE;


async function main() {
    const {api, chainName, getExplorerUrl} = await connect(RPC);
    log("Connected to blockchain:", chainName);

    const account = accountFromUri(SEED);
    log("From account", account.address);

    const bucketContract = getContract(BUCKET_CONTRACT_NAME, chainName, api);
    log("Using bucket contract", BUCKET_CONTRACT_NAME, "at", bucketContract.address.toString());

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
    const capacity = 1e6;
    const num_vnodes = 6;
    const cluster_resource = 10;
    const bucket_resource = 5;
    const bucket_params = "{}";

    let nodeId;
    {
        log("Setup a node…");
        const tx = bucketContract.tx
            .nodeCreate(txOptionsPay, rent_per_month, node_params, capacity);

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
        let cluster_params = "{}";
        const tx = bucketContract.tx
            .clusterCreate(txOptionsPay, managerId, num_vnodes, [nodeId], cluster_params);

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
            .clusterReserveResource(txOptions, clusterId, cluster_resource);

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
            .bucketCreate(txOptionsPay, bucket_params, clusterId);

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
            .bucketAllocIntoCluster(txOptions, bucketId, bucket_resource);

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

    // NFT registry
    const registryContract = getContract(NFT_REGISTRY_CONTRACT_NAME, chainName, api);
    log("Using nft registry contract", NFT_REGISTRY_CONTRACT_NAME, "at", registryContract.address.toString());

    // Test data.
    const nft_id = "0000000000000030ABCD1234ABCD1234ABCD1234ABCD1234ABCD12340000003132333435";
    const asset_id = "ddc:1234";
    const proof = "cere_tx";

    {
        log("Attach asset…");
        const tx = registryContract.tx.attach(txOptionsPay, nft_id, asset_id, proof);

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);


        const { nft_id, asset_id, proof} = ddcNftRegistry.findCreatedAttachment(events);
        log(`New attach: nft_id=${nft_id}, asset_id=${asset_id}, proof=${proof}\n`);
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
