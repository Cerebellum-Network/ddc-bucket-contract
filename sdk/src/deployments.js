const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0xa4ea89841ae87268bd86e98c3c3ef7d388f25ade162dd3750067bef524d35d4d");

    registerContract("ddc_bucket", "Cere Testnet", "5FwssT99zFF9a3jgCfAYUbsamha73sixzhUifobJSndEEptE");
}


module.exports = {
    init,
}
