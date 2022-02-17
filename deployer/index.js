const {ApiPromise, WsProvider} = require('@polkadot/api');
const {CodePromise, BlueprintPromise, ContractPromise} = require('@polkadot/api-contract');
const {Keyring} = require('@polkadot/api');
const fs = require("fs/promises");
const cereTypes = require("./cere_custom_types.json");

const log = console.log;

let CODE_HASH = "0xd7b513d798721f09f608068952eac05fb882790b49dcf7970f2eab6417eb57d8";
let CONTRACT_ADDRESS = "5FAKopbXL47wUdEp2s5sdBCg3vxFqg295M326bvbe84Zmd6N";

const WASM = "../target/ink/ddc_bucket/ddc_bucket.wasm";
const ABI = "../target/ink/ddc_bucket/metadata.json";
const CONSTRUCTOR = "new";

const SEED = "//Alice";
const RPC = "wss://rpc.devnet.cere.network:9945";
//const RPC = "wss://rococo-canvas-rpc.polkadot.io"; // Canvas

const showExplorerTx = (blockHash) => {
    log(`https://explorer.cere.network/?rpc=${RPC}#/explorer/query/${blockHash}`);
}

const CERE = 10_000_000_000n;
const MGAS = 1_000_000n;


async function main() {
    const wsProvider = new WsProvider(RPC);
    const api = await ApiPromise.create({
        provider: wsProvider,
        types: cereTypes,
    });
    log("runtimeChain:", api.runtimeChain.toString());

    const keyring = new Keyring({type: 'sr25519'});
    const account = keyring.addFromUri(SEED);
    log("From account", account.address);

    const abi = JSON.parse(await fs.readFile(ABI));
    log("ABI", abi.contract.name, abi.contract.version);

    const wasm = await fs.readFile(WASM);
    log("WASM", wasm.length, "bytes");


    async function sendTx(account, tx) {
        const result = await new Promise(async (resolve, reject) => {
            const unsub = await tx.signAndSend(account, (result) => {
                if (result.status.isInBlock || result.status.isFinalized) {
                    unsub();
                    resolve(result);
                }
            });
        });
        showExplorerTx(result.status.asInBlock.toString());
        return result;
    }


    // Upload code if necessary.
    if (!CODE_HASH) {
        log("Deploying the code…");

        // Construct our Code helper. The abi is an Abi object, an unparsed JSON string
        // or the raw JSON data (after doing a JSON.parse). The wasm is either a hex
        // string (0x prefixed), an Uint8Array or a Node.js Buffer object
        const code = new CodePromise(api, abi, wasm);
        const tx = code.tx[CONSTRUCTOR]({});

        // Deploy the WASM, retrieve a Blueprint
        const result = await sendTx(account, tx);
        CODE_HASH = result.blueprint.codeHash.toString();
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

        // We pass the constructor (named `new` in the actual Abi),
        // the endowment, gasLimit (weight) as well as any constructor params
        // (in this case `new (initValue: i32)` is the constructor)
        const blueprint = new BlueprintPromise(api, abi, CODE_HASH);
        const tx = blueprint.tx[CONSTRUCTOR](txOptions);

        // Instantiate the contract and retrieve its address.
        const result = await sendTx(account, tx);
        CONTRACT_ADDRESS = result.contract.address.toString();
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
        log("\nSending a transaction…");
        const tx = contract.tx
            .providerSetInfo(txOptions, 10n * CERE, "https://ddc.dev.cere.network/bucket/{BUCKET_ID}");

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
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
