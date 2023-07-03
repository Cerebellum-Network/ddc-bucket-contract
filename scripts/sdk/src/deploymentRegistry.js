const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");
const config = require("./config");
const ddcBucketAbi = require("./abi/ddc_bucket.json");


function initDefaultContracts() {

    registerABI(config.DDC_BUCKET_CONTRACT_NAME, ddcBucketAbi);

    registerContract(
        config.DDC_BUCKET_CONTRACT_NAME, 
        config.DEVNET_CHAIN_NAME, 
        config.DEVNET_DDC_BUCKET_ADDR
    );

    registerContract(
        config.DDC_BUCKET_CONTRACT_NAME, 
        config.TESTNET_CHAIN_NAME, 
        config.TESTNET_DDC_BUCKET_ADDR
    );

    registerContract(
        config.DDC_BUCKET_CONTRACT_NAME, 
        config.LOCAL_CHAIN_NAME, 
        config.LOCAL_DDC_BUCKET_ADDR
    );
    
}

function initContract(name, env, address) {
    registerABI(name, ddcBucketAbi);
    registerContract(
        name, 
        env,
        address
    );
}

module.exports = {
    initDefaultContracts,
    initContract
}
