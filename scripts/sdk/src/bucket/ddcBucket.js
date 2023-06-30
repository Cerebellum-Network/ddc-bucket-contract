const _ = require("lodash");


function findEvent(events, eventName) {
    const event = _.find(events, ["event.identifier", eventName]);
    if (!event) 
        throw new Error("Event '" + eventName + "' is not found");
    return event;
}

function findNodeCreatedEvent(events) {
    const event = findEvent(events, "NodeCreated");
    const nodeKey = _.get(event, "args[0]").toString();
    const providerId = _.get(event, "args[1]").toString();
    const rentPerMonth = _.get(event, "args[2]").toNumber();
    const nodeParams = _.get(event, "args[3]").toString();
    return { nodeKey, providerId, rentPerMonth, nodeParams };
}

function findCdnNodeCreatedEvent(events) {
    const event = findEvent(events, "CdnNodeCreated");
    const cdnNodeKey = _.get(event, "args[0]").toString();
    const providerId = _.get(event, "args[1]").toString();
    const cdnNodeParams = _.get(event, "args[2]").toString();
    const undistributedPayment = _.get(event, "args[3]").toNumber();
    return { cdnNodeKey, providerId, cdnNodeParams, undistributedPayment };
}

function findClusterCreatedEvent(events) {
    const event = findEvent(events, "ClusterCreated");
    const clusterId = _.get(event, "args[0]").toNumber();
    const managerId = _.get(event, "args[1]").toString();
    const clusterParams = _.get(event, "args[2]").toString();
    return { clusterId, managerId, clusterParams };
}

function findBucketCreatedEvent(events) {
    const event = findEvent(events, "BucketCreated");
    const bucketId = _.get(event, "args[0]").toNumber();
    const ownerId = _.get(event, "args[1]").toString();
    return { bucketId, ownerId };
}

function findPermissionGrantedEvent(events) {
    const event = findEvent(events, "PermissionGranted");
    const accountId = _.get(event, "args[0]").toString();
    const permission = _.get(event, "args[1]").toString();
    return { accountId, permission };
}

function findClusterReserveResourceEvent(events) {
    const event = findEvent(events, "ClusterReserveResource");
    const clusterId = _.get(event, "args[0]").toNumber();
    const reserved = _.get(event, "args[1]").toNumber();
    return { clusterId, reserved };
}

function findClusterNodeStatusSetEvent(events) {
    const event = findEvent(events, "ClusterNodeStatusSet");
    const nodeKey = _.get(event, "args[0]").toString();
    const clusterId = _.get(event, "args[1]").toNumber();
    const status = _.get(event, "args[2]").toString();
    return { nodeKey, clusterId, status };
}

function findClusterCdnNodeStatusSetEvent(events) {
    const event = findEvent(events, "ClusterCdnNodeStatusSet");
    const cdnNodeKey = _.get(event, "args[0]").toString();
    const clusterId = _.get(event, "args[1]").toNumber();
    const status = _.get(event, "args[2]").toString();
    return { cdnNodeKey, clusterId, status };
}

function findClusterNodeAddedEvent(events) {
    const event = findEvent(events, "ClusterNodeAdded");
    const clusterId = _.get(event, "args[0]").toNumber();
    const nodeKey = _.get(event, "args[1]").toString();
    return { nodeKey, clusterId };
}

function findClusterCdnNodeAddedEvent(events) {
    const event = findEvent(events, "ClusterCdnNodeAdded");
    const clusterId = _.get(event, "args[0]").toNumber();
    const cdnNodeKey = _.get(event, "args[1]").toString();
    return { cdnNodeKey, clusterId };
}

function findDepositEvent(events) {
    const event = findEvent(events, "Deposit");
    const accountId = _.get(event, "args[0]").toString();
    const value = _.get(event, "args[1]").toNumber();
    return { accountId, value };
}

function findBucketAllocatedEvent(events) {
    const event = findEvent(events, "BucketAllocated");
    const bucketId = _.get(event, "args[0]").toNumber();
    const clusterId = _.get(event, "args[1]").toNumber();
    const resource = _.get(event, "args[2]").toNumber();
    return { bucketId, clusterId, resource };
}

function findBucketSettlePaymentEvent(events) {
    const event = findEvent(events, "BucketSettlePayment");
    const bucketId = _.get(event, "args[0]").toNumber();
    const clusterId = _.get(event, "args[1]").toNumber();
    return { bucketId, clusterId };
}

function findClusterDistributeRevenuesEvent(events) {
    const event = findEvent(events, "ClusterDistributeRevenues");
    const clusterId = _.get(event, "args[0]").toNumber();
    const providerId = _.get(event, "args[1]").toString();
    return { clusterId, providerId };
}


module.exports = {
    findEvent,
    findNodeCreatedEvent,
    findCdnNodeCreatedEvent,
    findClusterCreatedEvent,
    findBucketCreatedEvent,
    findPermissionGrantedEvent,
    findClusterReserveResourceEvent,
    findClusterNodeStatusSetEvent,
    findClusterCdnNodeStatusSetEvent,
    findClusterNodeAddedEvent,
    findClusterCdnNodeAddedEvent,
    findDepositEvent,
    findBucketAllocatedEvent,
    findBucketSettlePaymentEvent,
    findClusterDistributeRevenuesEvent,
};
