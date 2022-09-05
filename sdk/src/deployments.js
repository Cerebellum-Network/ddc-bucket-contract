const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");


const ddc_bucket_abi = require("./abi/ddc_bucket.json");

function init() {
    registerABI("ddc_bucket", ddc_bucket_abi, "0xabfb40d2f3cd9742bdd462002e4b8629c375e941ba5d6a059e20b33e23871cec");

    // DDC DEVNET ADMIN: 5Fq4A47kApWUKP9CXZqbKvHMotQD5f4qeivLBM9CjZ1Da3GA
    registerContract("ddc_bucket", "Cere Devnet", "5EShbAjEMgwxrxM4MkE9tZYKd9du9LrW6F7wwin43UqgHZEV");

    // DDC TESTNET ADMIN: 5G27k5NH5p7qMDGuxkctUXAPw2HEf3Ef23TSByUAMomWb6U6
    registerContract("ddc_bucket", "Cere Testnet", "5D3yQXckkLAy7H7m37hVQdyYjXWtKs637pFFECD5hvB8YPaF");
}


module.exports = {
    init,
}
