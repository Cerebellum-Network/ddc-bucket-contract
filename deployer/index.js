const {ApiPromise, WsProvider} = require('@polkadot/api');
const {CodePromise, BlueprintPromise, ContractPromise} = require('@polkadot/api-contract');
const {Keyring} = require('@polkadot/api');
const fs = require("fs/promises");
const cereTypes = require("./cere_custom_types.json");

const log = console.log;

const DEVNET = "wss://rpc.devnet.cere.network:9945";
const CANVAS = "wss://rococo-canvas-rpc.polkadot.io";

const WASM = "../target/ink/cluster/cluster.wasm";
const ABI = "../target/ink/cluster/metadata.json";
const CONSTRUCTOR = "default";
let CODE_HASH = "0x4a59e9026e8b303d4943be62b9a6414956d0b7debdeb19282f0f4231b52ae4a0";
let CONTRACT_ADDRESS = "5GhChJ6TuNmWM9XkSftUoWcKr6xoV76jQcUgQwX1ULjcfSn4";

const SEED = "//Alice";

const CERE = 10000000000n;
const MGAS = 1000000n;

async function main() {
    const wsProvider = new WsProvider(DEVNET);
    const api = await ApiPromise.create({
        provider: wsProvider,
        types: cereTypes,
    });

    const keyring = new Keyring({type: 'sr25519'});
    const account = keyring.addFromUri(SEED);
    log("ACCOUNT", account.address);

    const abi = JSON.parse(await fs.readFile(ABI));
    //log("ABI", abi);

    const wasm = await fs.readFile(WASM);
    log("WASM", wasm.length, "bytes");

    // Upload code if necessary.
    if (!CODE_HASH) {
        log("Deploying the code…");

        // Deploy the WASM, retrieve a Blueprint
        const blueprint = await new Promise(async (resolve, reject) => {
            // Construct our Code helper. The abi is an Abi object, an unparsed JSON string
            // or the raw JSON data (after doing a JSON.parse). The wasm is either a hex
            // string (0x prefixed), an Uint8Array or a Node.js Buffer object
            const code = new CodePromise(api, abi, wasm);
            const ctor = code.tx[CONSTRUCTOR]({});

            const unsub = await ctor.signAndSend(account, (result) => {
                if (result.status.isInBlock || result.status.isFinalized) {
                    unsub();
                    resolve(result.blueprint);
                }
            });
        });
        CODE_HASH = blueprint.codeHash.toString();
        log("Deployed code", CODE_HASH);
    } else {
        log("Using existing code", CODE_HASH);
    }

    // Instantiate a new contract if necessary.
    if (!CONTRACT_ADDRESS) {
        log("Instantiating a contract…");

        const txOptions = {
            value: 10n * CERE,
            gasLimit: 100000n * MGAS,
        };

        const contract = await new Promise(async (resolve, reject) => {
            const blueprint = new BlueprintPromise(api, abi, CODE_HASH);

            // We pass the constructor (named `new` in the actual Abi),
            // the endowment, gasLimit (weight) as well as any constructor params
            // (in this case `new (initValue: i32)` is the constructor)
            const unsub = await blueprint.tx
                [CONSTRUCTOR](txOptions)
                .signAndSend(account, (result) => {
                    if (result.status.isInBlock || result.status.isFinalized) {
                        unsub();
                        resolve(result.contract);
                    }
                });
        });
        const CONTRACT_ADDRESS = contract.address.toString();
        log("Instantiated contract", CONTRACT_ADDRESS);
    } else {
        log("Using existing contract", CONTRACT_ADDRESS);
    }

    // Attach to an existing contract with a known ABI and address. As per the
    // code and blueprint examples the abi is an Abi object, an unparsed JSON
    // string or the raw JSON data (after doing a JSON.parse). The address is
    // the actual on-chain address as ss58 or AccountId object.
    const contract = new ContractPromise(api, abi, CONTRACT_ADDRESS);

    const txOptions = {
        value: 0n * CERE,
        gasLimit: 100000n * MGAS,
    };

    // Write
    {
        const result = await new Promise(async (resolve, reject) => {
            const unsub = await contract.tx
                .setPrice(txOptions, 11n * CERE)
                .signAndSend(account, (result) => {
                    if (result.status.isInBlock || result.status.isFinalized) {
                        unsub();
                        resolve(result);
                    }
                });
        });
        log("WRITE RESULT", JSON.stringify(result, null, 4));
    }

    // Read (no params at the end, for the `get` message)
    {
        const {gasConsumed, result, output} = await contract.query.getPrice(account.address, txOptions);

        // The actual result from RPC as `ContractExecResult`
        log("GET RESULT", result.toHuman());

        // gas consumed
        log("GAS CONSUMED", gasConsumed.toHuman());

        // check if the call was successful
        if (result.isOk) {
            log('OUTPUT', output.toHuman());
        } else {
            console.error('ERROR', result.asErr);
        }
    }
}

main().then(log, log);
