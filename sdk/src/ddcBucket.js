const _ = require("lodash");


function findCreatedId(events, eventName) {
    const event = _.find(events, ["event.identifier", eventName]);
    const bucketId = _.get(event, "args[0]");
    return bucketId.toString();
}

function findCreatedBucketId(events) {
    return findCreatedId(events, "BucketCreated");
}

function findCreatedDealId(events) {
    return findCreatedId(events, "DealCreated");
}


module.exports = {
    findCreatedId,
    findCreatedBucketId,
    findCreatedDealId,
};
