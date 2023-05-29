const {registerABI} = require("./abiRegistry.js");
const {registerContract} = require("./contractRegistry.js");
const config = require("./config");
const ddcBucketAbi = require("./abi/ddc_bucket.json");
const ddcNftRegistryAbi = require("./abi/ddc_nft_registry.json");


function initContracts() {

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


    registerABI(config.DDC_NFT_REGISTRY_CONTRACT_NAME, ddcNftRegistryAbi);

    registerContract(
        config.DDC_NFT_REGISTRY_CONTRACT_NAME, 
        config.DEVNET_CHAIN_NAME, 
        config.DEVNET_DDC_NFT_REGISTRY_ADDR
    );

    registerContract(
        config.DDC_NFT_REGISTRY_CONTRACT_NAME, 
        config.TESTNET_CHAIN_NAME, 
        config.TESTNET_DDC_NFT_REGISTRY_ADDR
    );

}


module.exports = {
    initContracts,
}
