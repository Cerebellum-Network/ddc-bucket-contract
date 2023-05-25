const {
    connect,
    accountFromUri,
    sendTx,
    getContract,
    CERE,
    MGAS,
    ddcNftRegistry,
    lodash: _,
    deploymentRegistry,
    config
} = require("./../sdk");

const log = console.log;

const NFT_REGISTRY_CONTRACT_NAME = config.DDC_NFT_REGISTRY_CONTRACT_NAME;
const SEED = config.ACTOR_SEED;
const RPC = config.DEVNET_RPC_ENDPOINT;

deploymentRegistry.initContracts();

async function main() {
    const {api, chainName, getExplorerUrl} = await connect(RPC);
    log("Connected to blockchain:", chainName);

    const account = accountFromUri(SEED);
    log("From account", account.address);

    const txOptions = {
        value: 0n,
        gasLimit: 200_000n * MGAS,
    };

    const txOptionsPay = {
        value: 10n * CERE,
        gasLimit: 200_000n * MGAS,
    };

    // NFT registry
    const registryContract = getContract(NFT_REGISTRY_CONTRACT_NAME, chainName, api);
    log("Using nft registry contract", NFT_REGISTRY_CONTRACT_NAME, "at", registryContract.address.toString());

    // Test data.
    const nftId = "0000000000000030ABCD1234ABCD1234ABCD1234ABCD1234ABCD12340000003132333435";
    const assetId = "ddc:1234";
    const proofVal = "cere_tx";

    {
        log("Attach asset…");
        const tx = registryContract.tx.attach(txOptionsPay, nftId, assetId, proofVal);

        const result = await sendTx(account, tx);
        printGas(result);
        log(getExplorerUrl(result));
        const events = printEvents(result);


        const { nft_id, asset_id, proof } = ddcNftRegistry.findCreatedAttachment(events);
        log(`New attach: nft_id=${nft_id}, asset_id=${asset_id}, proof=${proof}\n`);
    }

    process.exit(0);
}

function printEvents(result) {
    const events = result.contractEvents || [];
    //log("EVENTS", JSON.stringify(events, null, 4));
    log(events.length, "events");
    return events;
}

function printGas(result) {
    let gas = _.get(result, "dispatchInfo.weight", 0);
    log(parseInt(gas / 1e6), "MGas");
}

main().then(log, log);
