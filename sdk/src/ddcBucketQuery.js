const _ = require("lodash");

const TX_OPTIONS = {
    value: 0n,
    gasLimit: -1,
};
const ANY_ACCOUNT_ID = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
const LIMIT = 10;


async function listAll(contract, listMethod, optionalFilter) {
    let objects = [];

    let offset = 0;
    while (true) {
        let {result, output} = await contract.query
            [listMethod](ANY_ACCOUNT_ID, TX_OPTIONS, offset, LIMIT, optionalFilter);

        if (!result.isOk) throw result.asErr;
        let [more_objects, count] = output.toJSON();
        objects.push(...more_objects);

        offset += LIMIT;
        if (offset >= count) break;
    }

    return objects;
}


async function bucketListStatuses(contract, optionalFilter) {
    return listAll(contract, "bucketListStatuses", optionalFilter);
}

async function serviceList(contract, optionalFilter) {
    return listAll(contract, "serviceList", optionalFilter);
}


module.exports = {
    bucketListStatuses,
    serviceList,
};
