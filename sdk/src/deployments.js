const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0x26d05a5e69024ef58d014c92ae805fbe9c7302a637d56e60013f042ebd9ee7f6");

    registerContract("ddc_bucket", "Cere Testnet", "5FeGRnfvoBS2kb2wqrwvDYZWPiehErvEdZC14uPwfp9upcgE");
}


module.exports = {
    init,
}
