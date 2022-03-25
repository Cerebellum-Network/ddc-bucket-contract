const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0x0dc5d591b1b0288043b6c0ec3e28e82093e68cf5af8703c0c5e8208f3db93570");

    registerContract("ddc_bucket", "Cere Testnet", "5Coq3eUWxvc8MvEb151m8ymnNhLrUE32hC7zLGf1NLXPpLyN");
}


module.exports = {
    init,
}
