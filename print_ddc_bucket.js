const {
    connect,
    getContract,
    ddcBucketQuery,
} = require("./sdk");
const log = console.log;

const CONTRACT_NAME = "ddc_bucket";
const RPC = "wss://rpc.testnet.cere.network:9945";

const ACCOUNT_FILTER = null; // get data about all accounts.


async function main() {
    const {api, chainName} = await connect(RPC);
    log("Connected to blockchain:", chainName);

    const contract = getContract(CONTRACT_NAME, chainName, api);

    const clusters = await ddcBucketQuery.clusterList(contract, ACCOUNT_FILTER);
    log("\nClusters", JSON.stringify(clusters, null, 4));

    const nodes = await ddcBucketQuery.nodeList(contract, ACCOUNT_FILTER);
    log("\nNodes", JSON.stringify(nodes, null, 4));

    const buckets = await ddcBucketQuery.bucketList(contract, ACCOUNT_FILTER);
    log("\nBuckets", JSON.stringify(buckets, null, 4));

    printGraph(clusters, nodes, buckets);

    process.exit(0);
}


function printGraph(clusters, nodes, buckets) {
    log();
    log("```mermaid");
    log("graph BT;");
    log();

    for (status of nodes) {
        let {node_id, node} = status;
        // Define
        log(`Node_${node_id}[(Node ${node_id})]`);

        // Node to Provider.
        log(`Node_${node_id} -. owned by .-> Account_${node.provider_id.substr(0, 8)}`);
        log();
    }

    for (status of clusters) {
        let {cluster_id, cluster} = status;
        // Define
        log(`Cluster_${cluster_id}((Cluster ${cluster_id}))`);

        // Cluster to Manager.
        log(`Cluster_${cluster_id} -. managed by ..-> Account_${cluster.manager_id.substr(0, 8)}`);

        // Cluster to Node.
        for (i = 0; i < cluster.vnodes.length; i++) {
            let node_id = cluster.vnodes[i];
            log(`Cluster_${cluster_id} -- P${i} --> Node_${node_id}`);
        }
        log();
    }

    for (status of buckets) {
        let {bucket_id, bucket} = status;
        // Define
        log(`Bucket_${bucket_id}[[Bucket ${bucket_id}]]`);

        // Bucket to Owner.
        log(`Bucket_${bucket_id} -. owned by ...-> Account_${bucket.owner_id.substr(0, 8)}`);

        // Bucket to Cluster.
        log(`Bucket_${bucket_id} -- allocated into --> Cluster_${bucket.cluster_id}`);
        log();
    }

    log("```");
    log();
}

main().then(log, log);
