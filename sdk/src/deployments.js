const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0x840d4bfd5ee171934712531c6112083e5a07393608fa28fbca79e81ec3940262");

    registerContract("ddc_bucket", "Cere Testnet", "5HSPZToejRJwKPRRQSuhTJFN66kBo3bz18kkqJPUR5Jt6TF1");
}


module.exports = {
    init,
}
