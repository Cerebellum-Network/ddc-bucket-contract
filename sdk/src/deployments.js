const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0x7829cae3fad09b46cf757fb1acab51c47fbd6895360f51845d42c517d687a");

    registerContract("ddc_bucket", "Cere Testnet", "5G5z6G9if3LFGrPLNDJgcaKH6E2FYrEKbXVFNt86QM9yXsV9");
}


module.exports = {
    init,
}
