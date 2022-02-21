const _ = require("lodash");


function findCreatedBucketId(events) {
    const event = _.find(events, ["event.identifier", "BucketCreated"]);
    const bucketId = _.get(event, "args[0]");
    return bucketId.toString();
}


module.exports = {
    findCreatedBucketId,
};
