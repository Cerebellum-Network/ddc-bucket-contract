const _ = require("lodash");


function findCreatedId(events, eventName) {
    const event = _.find(events, ["event.identifier", eventName]);
    const id = _.get(event, "args[0]");
    return id && id.toString();
}

function findCreatedServiceId(events) {
    return findCreatedId(events, "ServiceCreated");
}

function findCreatedBucketId(events) {
    return findCreatedId(events, "BucketCreated");
}

function findCreatedDealId(events) {
    return findCreatedId(events, "DealCreated");
}


module.exports = {
    findCreatedId,
    findCreatedServiceId,
    findCreatedBucketId,
    findCreatedDealId,
};
