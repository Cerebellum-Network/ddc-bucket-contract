const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0xd90275a50ac34e3647c5aec6a000781b3dde58c4e1625827b5594b4974b91434");

    registerContract("ddc_bucket", "Cere Testnet", "5GKuEqBDHCiJw1KoiLrzjUqemtZhnd5B2GdmLtEiuaUtthG5");
}


module.exports = {
    init,
}
