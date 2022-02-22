const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0xc9e137e30ece8a464286e92cea70e4181ce3a80a2c4602da768e53632c5027d6");

    registerContract("ddc_bucket", "Cere Testnet", "5CNZigA12EL2LvE4DmQ5mNppjpJKumCTBx8suzZQEdL9rHQz");
}


module.exports = {
    init,
}
