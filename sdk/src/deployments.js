const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0x52fb51b7e7b89d4470a6356379bce342d5370566f5aedfd96016c1b958478a4b");

    registerContract("ddc_bucket", "Cere Testnet", "5E8FN8XvMg3jt3bfYBt34HMxuGYKMAjrsUaSRtCRaX2qwm9q");
}


module.exports = {
    init,
}
