const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0x04f51b74accd640aee7dcdf7f8872a614d1c45db20f0ea614c33b67c18ba95cf");

    registerContract("ddc_bucket", "Cere Testnet", "5EdUjmRwCtRCcXkAexZ5m5MEAXdFHFhNhdNTBKcbmEni3SnF");
}


module.exports = {
    init,
}
