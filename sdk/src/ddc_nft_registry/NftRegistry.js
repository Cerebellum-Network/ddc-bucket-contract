const _ = require("lodash");


function findByEventName(events, eventName) {
    const event = _.find(events, ["event.identifier", eventName]);
    const nft_id = _.get(event, "args[0]");
    const asset_id = _.get(event, "args[1]");
    const proof = _.get(event, "args[2]");
    return {
        nft_id, asset_id, proof
    };
}

function findCreatedAttachment(events) {
    return findByEventName(events, "Attach");
}


module.exports = {
    findCreatedAttachment,
};
