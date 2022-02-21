const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0xf880703598da623325b6521d31f59f78dc93598361173df3e31927258a32cff0");

    registerContract("ddc_bucket", "Cere Testnet", "5Frr6hqoMjKFsfANMd26qnGZhKXzbU1uooFgdTsWVLguwXDq");
}


module.exports = {
    init,
}
