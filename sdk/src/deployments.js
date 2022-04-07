const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0xdc6bcc300da19bf8d13f9da92addb3643079f722947e947f811487ae190331f6");

    registerContract("ddc_bucket", "Cere Testnet", "5HVvafs4xBbSnVDFSam9tArJrAFX8xnZALUn3cgDS5RDBufB");
}


module.exports = {
    init,
}
