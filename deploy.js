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

const DDC_CONTRACT_NAMES = [ "ddc_bucket", "ddc_nft_registry"]

const stash_data = {
    [DDC_CONTRACT_NAMES[0]]: {
        reuse_code_hash: "",
        reuse_contract_address: "",
    },
    [DDC_CONTRACT_NAMES[1]]: {
        reuse_code_hash: "",
        reuse_contract_address: "",
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
    const { abi_path , wasm_path } = getPaths(contract_name)
    const abi = JSON.parse(await fs.readFile(abi_path));
    log(`${contract_name} SC: ABI name=${abi.contract.name}, version=${abi.contract.version}`);

    const stashed = stash_data[contract_name]
    // Upload code if necessary.
    if (stashed.reuse_code_hash) {
        log("Using existing code", stashed.reuse_code_hash);
        registerABI(contract_name, abi, stashed.reuse_code_hash);
    } else {
        const wasm = await fs.readFile(wasm_path);
        log(`Deploying the code ${wasm_path} (${wasm.length} bytes)`);

        const code = getCodeDeployer(api, abi, wasm);
        const tx = code.tx.new(
            {  
                value: 0,
                gasLimit: 100_000n * MGAS,
                storageDepositLimit: 750_000_000_000 
            },
            ...[]
          );

        // Deploy the WASM, retrieve a Blueprint
        const result = await sendTx(account, tx);
        console.log(result);
        const new_code_hash = result.blueprint.codeHash.toString();
        log(getExplorerUrl(result));
        log("Deployed the code. Update stash_data in the script with new code hash:");
        log(`    const stash_data = {  `);
        log(`       ${contract_name}: {`);
        log(`           reuse_code_hash: ${new_code_hash}`);
        log(`       };`);
        registerABI(contract_name, abi, new_code_hash);
    }
    log();

    // Instantiate a new contract if necessary.
    if (stashed.reuse_contract_address) {
        log("Using existing contract", stashed.reuse_contract_address);
        registerContract(contract_name, chainName, stashed.reuse_contract_address);
    } else {
        log("Instantiating a contract…");

        const txOptions = {
            value: ENDOWMENT * 5n,
            gasLimit:  100_000n * MGAS,
            storageDepositLimit: 150000000000
        };

        const blueprint = getBlueprint(contract_name, api);
        const tx = blueprint.tx[CONSTRUCTOR](txOptions);

        // Instantiate the contract and retrieve its address.
        const result = await sendTx(account, tx);
        const new_contract_address = result.contract.address.toString();
        log(getExplorerUrl(result));
        log(`Instantiated a new contract. Update stash_data in the script with new contract address:`);
        log(`    const stash_data = {  `);
        log(`       ${contract_name}: {`);
        log(`           reuse_contract_address: ${new_contract_address}`);
        log(`       };`);

        registerContract(contract_name, chainName, new_contract_address);
    }
}

const getPaths = (contract_name) => {
    contract_name = contract_name.toLowerCase();
    const wasm_path = `./target/ink/${contract_name}/${contract_name}.wasm`;
    const abi_path = `./target/ink/${contract_name}/metadata.json`;
    return { wasm_path, abi_path }
}

main().then(log, log);
