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
let CODE_HASH = "0xf47dee0051665a329f8c8d8188a9fed9dc8d466cdaa90da838a3f8de8c649200";
let CONTRACT_ADDRESS = "5FiTtZN2ZmF2HqK7iLiPwVnxEtedNWX87ZUJFS6QhtvzqeY2";

const SEED = "//Alice";

const CERE = 10_000_000_000n;
const MGAS = 1_000_000n;

async function main() {
    const wsProvider = new WsProvider(DEVNET);
    const api = await ApiPromise.create({
        provider: wsProvider,
        types: cereTypes,
    });

    const keyring = new Keyring({type: 'sr25519'});
    const account = keyring.addFromUri(SEED);
    log("From account", account.address);

    const abi = JSON.parse(await fs.readFile(ABI));
    log("ABI", abi.contract.name, abi.contract.version);

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
            const tx = code.tx[CONSTRUCTOR]({});

            const unsub = await tx.signAndSend(account, (result) => {
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
            gasLimit: 100_000n * MGAS,
        };

        const contract = await new Promise(async (resolve, reject) => {
            const blueprint = new BlueprintPromise(api, abi, CODE_HASH);

            // We pass the constructor (named `new` in the actual Abi),
            // the endowment, gasLimit (weight) as well as any constructor params
            // (in this case `new (initValue: i32)` is the constructor)
            const tx = blueprint.tx[CONSTRUCTOR](txOptions);

            const unsub = await tx.signAndSend(account, (result) => {
                if (result.status.isInBlock || result.status.isFinalized) {
                    unsub();
                    resolve(result.contract);
                }
            });
        });
        CONTRACT_ADDRESS = contract.address.toString();
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
        gasLimit: -1, //100_000n * MGAS,
    };

    // Write
    {
        const tx = contract.tx
            .setLocation(txOptions, "https://abc");

        const result = await new Promise(async (resolve, reject) => {
            const unsub = await tx.signAndSend(account, (result) => {
                if (result.status.isInBlock || result.status.isFinalized) {
                    unsub();
                    resolve(result);
                }
            });
        });

        const events = result.contractEvents || [];
        log("TX in block", result.status.asInBlock.toString());
        log("EVENTS", JSON.stringify(events, null, 4));
    }

    // Read (no params at the end, for the `get` message)
    {
        const {gasConsumed, result, output} = await contract.query
            .getLocation(account.address, txOptions);

        if (result.isOk) {
            log('OUTPUT', output.toHuman());
        } else {
            console.error('ERROR', result.asErr);
        }

        //log("GET RESULT", result.toHuman());
        //log("GAS CONSUMED", gasConsumed.toHuman());
    }
}

main().then(log, log);
