const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0x5e690c8fd199cbf15e7f27bf1b510235aaf5d2bfebc776f72b77f04e03c1ad3b");

    registerContract("ddc_bucket", "Cere Testnet", "5DAx9cTNXYKbbMTQUWzh1cZ46Mj14pnyKvshkVWm8fkfh36X");
}


module.exports = {
    init,
}
