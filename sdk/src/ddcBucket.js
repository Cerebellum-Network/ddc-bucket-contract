const _ = require("lodash");


function findCreatedId(events, eventName) {
    const event = _.find(events, ["event.identifier", eventName]);
    const id = _.get(event, "args[0]");
    return id && id.toString();
}

function findCreatedVNodeId(events) {
    return findCreatedId(events, "VNodeCreated");
}

function findCreatedClusterId(events) {
    return findCreatedId(events, "ClusterCreated");
}

function findCreatedBucketId(events) {
    return findCreatedId(events, "BucketCreated");
}

function findCreatedDealId(events) {
    return findCreatedId(events, "DealCreated");
}


module.exports = {
    findCreatedId,
    findCreatedVNodeId,
    findCreatedClusterId,
    findCreatedBucketId,
    findCreatedDealId,
};
