const {
    connect,
    accountFromUri,
    registerABI,
    getCodeDeployer,
    sendTx,
    registerContract,
    getBlueprint,
    getContract,
    CERE,
    MGAS,
    ddcBucket,
} = require("./sdk");

const fs = require("fs/promises");
const assert = require("assert");
const log = console.log;

const CONTRACT_NAME = "ddc_bucket";
const REUSE_CODE_HASH = "";
const REUSE_CONTRACT_ADDRESS = "";

const WASM = `./target/ink/${CONTRACT_NAME}/${CONTRACT_NAME}.wasm`;
const ABI = `./target/ink/${CONTRACT_NAME}/metadata.json`;
const CONSTRUCTOR = "new";

const SEED = "//Alice";
const RPC = "wss://rpc.testnet.cere.network:9945";

const ENDOWMENT = 10n * CERE;


async function main() {
    const {api, chainName, getExplorerUrl} = await connect(RPC);
    log("Connected to blockchain:", chainName);

    const account = accountFromUri(SEED);
    log("From account", account.address);

    const abi = JSON.parse(await fs.readFile(ABI));
    log("ABI", abi.contract.name, abi.contract.version);

    // Upload code if necessary.
    if (REUSE_CODE_HASH) {
        log("Using existing code", REUSE_CODE_HASH);
        registerABI(CONTRACT_NAME, abi, REUSE_CODE_HASH);
    } else {
        const wasm = await fs.readFile(WASM);
        log(`Deploying the code ${WASM} (${wasm.length} bytes)`);

        const code = getCodeDeployer(api, abi, wasm);
        const tx = code.tx[CONSTRUCTOR]({});

        // Deploy the WASM, retrieve a Blueprint
        const result = await sendTx(account, tx);
        const new_code_hash = result.blueprint.codeHash.toString();
        log(getExplorerUrl(result));
        log("Deployed the code. Write this in the script:");
        log(`    const REUSE_CODE_HASH = "${new_code_hash}";`);
        registerABI(CONTRACT_NAME, abi, new_code_hash);
    }
    log();

    // Instantiate a new contract if necessary.
    if (REUSE_CONTRACT_ADDRESS) {
        log("Using existing contract", REUSE_CONTRACT_ADDRESS);
        registerContract(CONTRACT_NAME, chainName, REUSE_CONTRACT_ADDRESS);
    } else {
        log("Instantiating a contract…");

        const txOptions = {
            value: ENDOWMENT,
            gasLimit: 100_000n * MGAS,
        };

        const blueprint = getBlueprint(CONTRACT_NAME, api);
        const tx = blueprint.tx[CONSTRUCTOR](txOptions);

        // Instantiate the contract and retrieve its address.
        const result = await sendTx(account, tx);
        const new_contract_address = result.contract.address.toString();
        log(getExplorerUrl(result));
        log("Instantiated a new contract. Write this in the script:");
        log(`    const REUSE_CONTRACT_ADDRESS = "${new_contract_address}";`);
        registerContract(CONTRACT_NAME, chainName, new_contract_address);
    }

    process.exit(0);
}

main().then(log, log);
