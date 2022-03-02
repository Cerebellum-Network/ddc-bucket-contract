const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0x75f300198e48015c82edf1eade7f8eab198608a9cbe08a950920c3aa834d834d");

    registerContract("ddc_bucket", "Cere Testnet", "5EjAx9r2VwMAJDg5zgbUCSfcZNKRz8p7ZoWGdsyjEkvnFYY8");
}


module.exports = {
    init,
}
