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
} = require("./sdk");

const fs = require("fs/promises");
const log = console.log;

const CONTRACT_NAME = "ddc_bucket";
const REUSE_CODE_HASH = "0xd7b513d798721f09f608068952eac05fb882790b49dcf7970f2eab6417eb57d8";
const REUSE_CONTRACT_ADDRESS = "5FAKopbXL47wUdEp2s5sdBCg3vxFqg295M326bvbe84Zmd6N";

const WASM = `../target/ink/${CONTRACT_NAME}/${CONTRACT_NAME}.wasm`;
const ABI = `../target/ink/${CONTRACT_NAME}/metadata.json`;
const CONSTRUCTOR = "new";

const SEED = "//Alice";
const RPC = "wss://rpc.devnet.cere.network:9945";


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
            value: 10n * CERE,
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
    log();

    const contract = getContract(CONTRACT_NAME, chainName, api);

    const txOptions = {
        value: 0n * CERE,
        gasLimit: -1, //100_000n * MGAS,
    };

    // Write
    {
        log("Sending a transaction…");
        const tx = contract.tx
            .providerSetInfo(txOptions, 10n * CERE, "https://ddc.dev.cere.network/bucket/{BUCKET_ID}");

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
    }

    // Read (no params at the end, for the `get` message)
    {
        log("\nReading…");
        const {result, output} = await contract.query
            .providerGetInfo(account.address, txOptions, account.address);

        if (result.isOk) {
            log('OUTPUT', output.toHuman());
        } else {
            console.error('ERROR', result.asErr);
        }
    }
}

main().then(log, log);
