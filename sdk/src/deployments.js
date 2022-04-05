const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0xdf34b3cb3607cadac49e363a787dfb1e74b47b71af0e375a4eb89f33c1168edf");

    registerContract("ddc_bucket", "Cere Testnet", "5DHey2c1XfgCjiVLddwG46mhCUMSw7iSL2a8TxRZR1SjezPX");
}


module.exports = {
    init,
}
