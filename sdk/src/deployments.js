const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi);

    registerContract("ddc_bucket", "Cere Devnet", "5FAKopbXL47wUdEp2s5sdBCg3vxFqg295M326bvbe84Zmd6N");
}


module.exports = {
    init,
}
