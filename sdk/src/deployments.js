const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0x88c54aa08d5e07ea623486675b4bd471c6d0f34d481efecd2089a7ff1aba6f6a");

    registerContract("ddc_bucket", "Cere Testnet", "5GX6o5uijAYy1h5NSMfpYFeoEUb2iAnepgqgmjyqFsURoxNE");
}


module.exports = {
    init,
}
