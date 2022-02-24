const {
    connect,
    getContract,
    ddcBucketQuery,
} = require("./sdk");
const log = console.log;

const CONTRACT_NAME = "ddc_bucket";
const RPC = "wss://rpc.testnet.cere.network:9945";


async function main() {
    const {api, chainName} = await connect(RPC);
    log("Connected to blockchain:", chainName);

    const contract = getContract(CONTRACT_NAME, chainName, api);

    const services = await ddcBucketQuery.serviceList(contract);
    log("\nServices", JSON.stringify(services, null, 4));

    const buckets = await ddcBucketQuery.bucketListStatuses(contract);
    log("\nBuckets", JSON.stringify(buckets, null, 4));


    process.exit(0);
}

main().then(log, log);