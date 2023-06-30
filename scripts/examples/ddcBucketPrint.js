const {
    connect,
    getContract,
    ddcBucketQuery,
    lodash: _,
    deploymentRegistry,
    config
} = require("./../sdk");
const log = console.log;


const CONTRACT_NAME = config.DDC_BUCKET_CONTRACT_NAME;
const RPC = config.DEVNET_RPC_ENDPOINT;
const ACCOUNT_FILTER = null; // get data about all accounts.

deploymentRegistry.initDefaultContracts();

async function main() {
    const {api, chainName} = await connect(RPC);
    const contract = getContract(CONTRACT_NAME, chainName, api);
    log("Using contract", contract.address.toString(), "on", chainName);

    const clusters = await ddcBucketQuery.clusterList(contract, ACCOUNT_FILTER);
    log("\n## Clusters\n");
    printTable(clusters);
    //log(JSON.stringify(clusters, null, 4));

    const nodes = await ddcBucketQuery.nodeList(contract, ACCOUNT_FILTER);
    log("\n## Nodes\n");
    printTable(nodes);
    //log(JSON.stringify(nodes, null, 4));

    const buckets = await ddcBucketQuery.bucketList(contract, ACCOUNT_FILTER);
    log("\n## Buckets\n");
    printTable(buckets);
    //log(JSON.stringify(buckets, null, 4));

    log("\n## Network Graph\n");
    printGraph(clusters, nodes, buckets);

    process.exit(0);
}


function printTable(entities) {
    if (!entities.length) return;

    let header = getHeader(entities[0]);
    printHeader(header);
    for (entity of entities) {
        printEntity(header, entity);
    }
}

function getHeader(entity) {
    let header = [];
    for (k of _.keys(entity)) {
        let v = entity[k];
        if (_.isObject(v) && !_.isArray(v)) {
            let deepKeys = _.keys(v);
            for (dk of deepKeys) {
                header.push(k + "." + dk);
            }
        } else {
            header.push(k);
        }
    }
    return header;
}

function printHeader(header) {
    let names = _.map(header, (path) => {
        let parts = path.split(".");
        return parts[parts.length - 1];
    });
    let separator = _.map(header, () => "---");
    printRow(names);
    printRow(separator);
}

function printEntity(header, entity) {
    let row = [];
    for (k of header) {
        let v = _.get(entity, k);
        if (_.isObject(v)) {
            fixEntity(v);
            v = JSON.stringify(v).substr(0, 200);
        }
        row.push(v);
    }
    printRow(row);
}

// Fix some strange data that does not render well.
function fixEntity(entity) {
    if (entity.schedule) entity.schedule = "…";
}

function printRow(row) {
    let line = "| " + row.join("\t| ") + " |";
    log(line);
}


function printGraph(clusters, nodes, buckets) {
    log("```mermaid");
    log("graph BT;");
    log();

    for (status of nodes) {
        let {nodeKey, node} = status;
        // Define
        log(`Node_${nodeKey}[(Node ${nodeKey})]`);

        // Node to Provider.
        log(`Node_${nodeKey} -. owned by .-> Account_${node.providerId.substr(0, 8)}`);
        log();
    }

    for (status of clusters) {
        let {clusterId, cluster, clusterVNodes} = status;
        // Define
        log(`Cluster_${clusterId}((Cluster ${clusterId}))`);

        // Cluster to Manager.
        log(`Cluster_${clusterId} -. managed by ..-> Account_${cluster.managerId.substr(0, 8)}`);

        // Cluster to Node.
        for (i = 0; i < clusterVNodes.length; i++) {
            let vNodeId = clusterVNodes[i];
            log(`Cluster_${clusterId} -- P${i} --> Node_${vNodeId}`);
        }
        log();
    }

    for (status of buckets) {
        let {bucketId, bucket} = status;

        log(`Bucket_${bucketId}[[Bucket ${bucketId}]]`);

        // Bucket to Owner.
        log(`Bucket_${bucketId} -. owned by ...-> Account_${bucket.ownerId.substr(0, 8)}`);

        // Bucket to Cluster.
        log(`Bucket_${bucketId} -- allocated into --> Cluster_${bucket.clusterId}`);
        log();
    }

    log("```");
    log();
}

main().then(log, log);
