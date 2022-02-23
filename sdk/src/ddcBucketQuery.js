const _ = require("lodash");

const TX_OPTIONS = {
    value: 0n,
    gasLimit: -1,
};
const ANY_ACCOUNT_ID = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
const LIMIT = 1;


async function bucketListStatuses(contract) {
    let buckets = [];

    let offset = 0;
    while (true) {
        let {result, output} = await contract.query
            .bucketListStatuses(ANY_ACCOUNT_ID, TX_OPTIONS, offset, LIMIT);

        if (!result.isOk) throw result.asErr;
        let [more_buckets, count] = output.toJSON();
        buckets.push(...more_buckets);

        offset += LIMIT;
        if (offset >= count) break;
    }

    return buckets;
}


module.exports = {
    bucketListStatuses,
};
