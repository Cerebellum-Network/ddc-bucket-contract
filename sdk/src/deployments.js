const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0x5900876f0e07782c85b624fe8a85865e3103bbdc9b50bb676a8eafdf97234472");

    registerContract("ddc_bucket", "Cere Testnet", "5GrS5krqZf1cKkofUmoKeU2rZ3XcMTpG3Qfk2EUDWWPemsy4");
}


module.exports = {
    init,
}
