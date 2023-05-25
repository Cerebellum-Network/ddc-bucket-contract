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
    config
} = require("./sdk");

const fs = require("fs/promises");
const assert = require("assert");
const log = console.log;

const DDC_CONTRACT_NAMES = ["ddc_bucket", "ddc_nft_registry"]

const stash_data = {
    [DDC_CONTRACT_NAMES[0]]: {
        reuse_code_hash: ""
    },
    [DDC_CONTRACT_NAMES[1]]: {
        reuse_code_hash: ""
    }
}

const CONSTRUCTOR = "new";

const SEED = config.seed;
const RPC = config.rpc;

const ENDOWMENT = CERE;


async function main() {
    const {api, chainName, getExplorerUrl} = await connect(RPC);
    log("Connected to blockchain:", chainName);

    const account = accountFromUri(SEED);
    log("From account", account.address);

    for (const i in DDC_CONTRACT_NAMES) {
        await deployContract(DDC_CONTRACT_NAMES[i], api, account, chainName, getExplorerUrl)
    }

    process.exit(0);
}

const deployContract = async (contract_name, api, account, chainName, getExplorerUrl) => {
    const abi_path = getAbiPath(contract_name)
    const abi = JSON.parse(await fs.readFile(abi_path));
    log(`${contract_name} SC: ABI name=${abi.contract.name}, version=${abi.contract.version}`);

    const stashed = stash_data[contract_name]

    if (!stashed.reuse_code_hash) {
        // Deploy the WASM code and instantinate the contract
        const wasm_path = getWasmPath(contract_name);
        const wasm = await fs.readFile(wasm_path);
        log(`Deploying the WASM code ${wasm_path} (${wasm.length} bytes)`);

        const code = getCodeDeployer(api, abi, wasm);
        const tx = code.tx.new(
            {  
                value: 0,
                gasLimit: 100_000n * MGAS,
                storageDepositLimit: 750_000_000_000 
            },
            ...[]
          );

        const result = await sendTx(account, tx);
        const { contract, blueprint } = result;

        if (contract && blueprint == null) {
            const existing_code_hash = contract.abi.json.source.hash;
            const new_contract_address = result.contract.address.toString();

            log(getExplorerUrl(result));
            log("Using the existing WASM code that was uploaded previously. Update stash_data in the script with existing code hash:");
            log(`    const stash_data = {  `);
            log(`       ${contract_name}: {`);
            log(`           reuse_code_hash: ${existing_code_hash}`);
            log(`       };`);

            registerABI(contract_name, abi, existing_code_hash);
            registerContract(contract_name, chainName, new_contract_address);

        } else if (contract) {
            const new_code_hash = blueprint.codeHash.toString();
            const new_contract_address = result.contract.address.toString();

            log(getExplorerUrl(result));
            log("New WASM code has been deployed. Update stash_data in the script with new code hash:");
            log(`    const stash_data = {  `);
            log(`       ${contract_name}: {`);
            log(`           reuse_code_hash: ${new_code_hash}`);
            log(`       };`);

            registerABI(contract_name, abi, new_code_hash);
            registerContract(contract_name, chainName, new_contract_address);

        } else {
            log(getExplorerUrl(result));
            log("Code deployment failed. Make sure your account has enough funds or check for other errors using the link above");
        }

    } else {
        // Instantinate the contract from previously uploaded WASM code
        log("Instantiating a contract from previously uploaded WASM code");

        const txOptions = {
            value: ENDOWMENT * 5n,
            gasLimit:  100_000n * MGAS,
            storageDepositLimit: 150000000000
        };

        const blueprint = getBlueprint(contract_name, api, stashed.reuse_code_hash);
        const tx = blueprint.tx[CONSTRUCTOR](txOptions);

        // Instantiate the contract and retrieve its address.
        const result = await sendTx(account, tx);
        const { contract } = result;

        const new_contract_address = contract.address.toString();
        log(getExplorerUrl(result));
        log(`Instantiated a new contract. Update stash_data in the script with new contract address:`);
        log(`    const stash_data = {  `);
        log(`       ${contract_name}: {`);
        log(`           reuse_contract_address: ${new_contract_address}`);
        log(`       };`);

        registerABI(contract_name, abi, stashed.reuse_code_hash);
        registerContract(contract_name, chainName, new_contract_address);
    }
}

const getAbiPath = (contract_name) => {
    contract_name = contract_name.toLowerCase();
    const abi_path = `./target/ink/${contract_name}/metadata.json`;
    return abi_path;
}

const getWasmPath = (contract_name) => {
    contract_name = contract_name.toLowerCase();
    const wasm_path = `./target/ink/${contract_name}/${contract_name}.wasm`;
    return wasm_path;
}

main().then(log, log);
