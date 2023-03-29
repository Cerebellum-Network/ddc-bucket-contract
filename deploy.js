// const {
//     connect,
//     accountFromUri,
//     registerABI,
//     getCodeDeployer,
//     sendTx,
//     registerContract,
//     getBlueprint,
//     getContract,
//     CERE,
//     MGAS,
//     ddcBucket,
// } = require("./sdk");

// const fs = require("fs/promises");
// const assert = require("assert");
// const log = console.log;

// const DDC_CONTRACT_NAMES = [ "ddc_bucket", "ddc_nft_registry"]

// const stash_data = {
//     [DDC_CONTRACT_NAMES[0]]: {
//         reuse_code_hash: "",
//         reuse_contract_address: "",
//     },
//     [DDC_CONTRACT_NAMES[1]]: {
//         reuse_code_hash: "",
//         reuse_contract_address: "",
//     }
// }

// const CONSTRUCTOR = "new";

// const SEED = "//Alice";
// const RPC = "wss://rpc.testnet.cere.network:9945";

// const ENDOWMENT = 10n * CERE;


// async function main() {
//     const {api, chainName, getExplorerUrl} = await connect(RPC);
//     log("Connected to blockchain:", chainName);

//     const account = accountFromUri(SEED);
//     log("From account", account.address);

//     for (const sc in DDC_CONTRACT_NAMES) {
//         await deployContract(sc, api, account, chainName, getExplorerUrl)
//     }

//     process.exit(0);
// }

// const deployContract = async (contract_name, api, account, chainName, getExplorerUrl) => {
//     const { abi_path , wasm_path } = getPaths(contract_name)
//     const abi = JSON.parse(await fs.readFile(abi_path));
//     log(`${contract_name} SC: ABI name=${abi.contract.name}, version=${abi.contract.version}`);

//     const stashed = stash_data[contract_name]
//     // Upload code if necessary.
//     if (stashed.reuse_code_hash) {
//         log("Using existing code", stashed.reuse_code_hash);
//         registerABI(contract_name, abi, stashed.reuse_code_hash);
//     } else {
//         const wasm = await fs.readFile(wasm_path);
//         log(`Deploying the code ${wasm_path} (${wasm.length} bytes)`);

//         const code = getCodeDeployer(api, abi, wasm);
//         const tx = code.tx[CONSTRUCTOR]({});

//         // Deploy the WASM, retrieve a Blueprint
//         const result = await sendTx(account, tx);
//         const new_code_hash = result.blueprint.codeHash.toString();
//         log(getExplorerUrl(result));
//         log("Deployed the code. Update stash_data in the script with new code hash:");
//         log(`    const stash_data = {  `);
//         log(`       ${contract_name}: {`);
//         log(`           reuse_code_hash: ${new_code_hash}`);
//         log(`       };`);
//         registerABI(contract_name, abi, new_code_hash);
//     }
//     log();

//     // Instantiate a new contract if necessary.
//     if (stashed.reuse_contract_address) {
//         log("Using existing contract", stashed.reuse_contract_address);
//         registerContract(contract_name, chainName, stashed.reuse_contract_address);
//     } else {
//         log("Instantiating a contract…");

//         const txOptions = {
//             value: ENDOWMENT,
//             gasLimit: 100_000n * MGAS,
//         };

//         const blueprint = getBlueprint(contract_name, api);
//         const tx = blueprint.tx[CONSTRUCTOR](txOptions);

//         // Instantiate the contract and retrieve its address.
//         const result = await sendTx(account, tx);
//         const new_contract_address = result.contract.address.toString();
//         log(getExplorerUrl(result));
//         log(`Instantiated a new contract. Update stash_data in the script with new contract address:`);
//         log(`    const stash_data = {  `);
//         log(`       ${contract_name}: {`);
//         log(`           reuse_contract_address: ${new_contract_address}`);
//         log(`       };`);

//         registerContract(contract_name, chainName, new_contract_address);
//     }
// }

// const getPaths = (contract_name) => {
//     contract_name = contract_name.toLowerCase();
//     const wasm_path = `./target/ink/${contract_name}/${contract_name}.wasm`;
//     const abi_path = `./target/ink/${contract_name}/metadata.json`;
//     return { wasm_path, abi_path }
// }



// main().then(log, log);
const { Keyring } = require("@polkadot/keyring");
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { CodePromise, Abi } = require("@polkadot/api-contract");
const { readFile } = require("fs/promises");

const cereTypes = {
    ChainId: "u8",
    DepositNonce: "u64",
    ResourceId: "[u8; 32]",
    ProposalStatus: {
        _enum: ["Initiated", "Approved", "Rejected"],
    },
    ProposalVotes: {
        votes_for: "Vec<AccountId>",
        votes_against: "Vec<AccountId>",
        status: "ProposalStatus",
        expiry: "BlockNumber",
    },
    TokenId: "u256",
    Erc721Token: {
        id: "TokenId",
        metadata: "Vec<u8>",
    },
    Address: "MultiAddress",
    LookupSource: "MultiAddress",
};

const accounts = {
    alice: "//Alice",
    bob: "//Bob",
    sudo: "book sunny rice catalog order replace stove exclude hollow muffin glory fame",
};

async function signAndSendPromise(
    txn,
    signer
) {
    return new Promise((res, rej) => {
        txn
            .signAndSend(signer, ({ events = [], status }) => {
                if (status.isInvalid) {
                    console.info("Transaction invalid");
                    rej("Transaction invalid");
                } else if (status.isReady) {
                    console.info("Transaction is ready");
                } else if (status.isBroadcast) {
                    console.info("Transaction has been broadcasted");
                } else if (status.isInBlock) {
                    const blockHash = status.asInBlock.toHex();
                    console.info(`Transaction is in block: ${blockHash}`);
                } else if (status.isFinalized) {
                    const blockHash = status.asFinalized.toHex();
                    console.info(
                        `Transaction has been included in blockHash ${blockHash}`
                    );
                    const treasuryDeposit = events.find(
                        (event) =>
                            event.event.toHuman().method === "Deposit" &&
                            event.event.toHuman().section === "treasury"
                    );
                    const txFee = treasuryDeposit
                        ? treasuryDeposit.event.toHuman().data[0]
                        : undefined;
                    const txFeeParsed = txFee
                        ? (parseFloat(txFee.replace(" mCERE", "")) / 1000) * 2
                        : undefined;
                    res({ blockHash, txFeeParsed, events });
                }
            })
            .catch((err) => rej(err));
    });
}



async function main() {
    const cereUrl = "ws://127.0.0.1:9944";
    // const cereUrl = "wss://archive.qanet.cere.network/ws";
    console.log(`Connecting to blockchain: ${cereUrl}\n`);

    const wsProvider = new WsProvider(cereUrl);

    const api = await ApiPromise.create({
        provider: wsProvider,
        types: cereTypes,
    });

    await api.isReady;
    const chain = await api.rpc.system.chain();

    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri(accounts.alice);

    console.log(alice)
    console.log(alice.address)

    console.log(`Connected to: ${chain}\n`);

    const abiPath = "./target/ink/ddc_bucket/metadata.json"
    const wasmPath = "./target/ink/ddc_bucket/ddc_bucket.contract"



    const abi = JSON.parse(await readFile(abiPath))
    const wasm = JSON.parse((await readFile(wasmPath)).toString()).source.wasm

    const gasLimit = 100000n * 1000000n;
    const storageDepositLimit = null;

    const codePromise = new CodePromise(api, abi, wasm);

    const tx = codePromise.tx.new({ gasLimit, storageDepositLimit });

    const result = await signAndSendPromise(tx, alice);

    const contractInstantiatedEvent = result.events.find(
        (event) =>
            event.event.toHuman().section === "contracts" &&
            event.event.toHuman().method === "Instantiated"
    );
    const contractAddress = contractInstantiatedEvent
        ? contractInstantiatedEvent.event.toHuman().data[1]
        : undefined;

    console.log(
        `Deploy DDC bucket smart contract test completed successfully. DDC bucket SC address is '${contractAddress}'`
    );
}

main().then(
    x => { process.exit(0) },
    e => console.log
);