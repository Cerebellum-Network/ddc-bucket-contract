const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0x4979b0025b64cd25c2a7fa4a7e4299fd9845d25ec7ac3e53562b5e4050d63334");

    registerContract("ddc_bucket", "Cere Testnet", "5DvZVawJJaaVLL9d1wAfrqc6v1RUmx5Yan8YuVYvvFKmgoFR");
}


module.exports = {
    init,
}
