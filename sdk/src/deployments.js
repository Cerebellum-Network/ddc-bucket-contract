const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0xec3a60f3fff9ee251277724e84ce2a13ed8d2bedd1e7e3f3640f56e85e98233d");

    registerContract("ddc_bucket", "Cere Testnet", "5FDGE7j9DXDbh6TmHkz2bR8FsN3VbSGjCQgbWzSc3izBqdXW");
}


module.exports = {
    init,
}
