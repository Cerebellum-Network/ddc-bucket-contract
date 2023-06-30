const {
    connect,
    accountFromUri,
    registerABI,
    getCodeDeployer,
    sendTx,
    registerContract,
    getBlueprint,
    MGAS,
    deploymentRegistry,
    config
} = require("./../sdk");
const fs = require("fs/promises");
const log = console.log;


const SEED = config.ACTOR_SEED;
const RPC = config.DEVNET_RPC_ENDPOINT;

const deployContract = async (
    contractName, 
    maybeCodeHash = null, 
    constructorName = "new"
) => {

    const {api, chainName, getExplorerUrl} = await connect(RPC);
    log("Connected to blockchain:", chainName);

    const account = accountFromUri(SEED);
    log("From account", account.address);

    if (maybeCodeHash) {
        deploymentRegistry.initDefaultContracts();
    }

    await deploy(
        contractName, 
        maybeCodeHash, 
        constructorName, 
        api, 
        account, 
        chainName, 
        getExplorerUrl
    );
}

const deploy = async (
    contractName, 
    maybeCodeHash,
    constructorName,
    api, 
    account, 
    chainName, 
    getExplorerUrl
) => {

    const abiPath = getAbiPath(contractName)
    const abi = JSON.parse(await fs.readFile(abiPath));
    log(`${contractName} SC: ABI name=${abi.contract.name}, version=${abi.contract.version}`);

    if (!maybeCodeHash) {
        // Deploy the WASM code and instantinate the contract
        const wasmPath = getWasmPath(contractName);
        const wasm = await fs.readFile(wasmPath);
        log(`Deploying the WASM code ${wasmPath} (${wasm.length} bytes)`);

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
            const existingCodeHash = contract.abi.json.source.hash;
            const newContractAddress = result.contract.address.toString();

            log(getExplorerUrl(result));
            log(`The existing WASM code was used.`);
            log(`New ${contractName} contract with address ${newContractAddress} is instantiated from existing wasm code with hash ${existingCodeHash}`);

            registerABI(contractName, abi, existingCodeHash);
            registerContract(contractName, chainName, newContractAddress);

        } else if (contract) {
            const newCodeHash = blueprint.codeHash.toString();
            const newContractAddress = contract.address.toString();

            log(getExplorerUrl(result));
            log(`New WASM code has been uploaded.`);
            log(`New ${contractName} contract with address ${newContractAddress} is instantiated from the uploaded code with hash ${newCodeHash}`)

            registerABI(contractName, abi, newCodeHash);
            registerContract(contractName, chainName, newContractAddress);

        } else {
            log(getExplorerUrl(result));
            log("Code deployment failed. Make sure your account has enough funds or check for other errors using the link above");
        }

    } else {
        log(`Instantiating new ${contractName} contract with previously uploaded WASM code`);

        const txOptions = {
            value: 0,
            gasLimit:  100_000n * MGAS,
            storageDepositLimit: 750_000_000_000
        };

        const blueprint = getBlueprint(contractName, api, maybeCodeHash);
        const tx = blueprint.tx[constructorName](txOptions);

        // Instantiate the contract and retrieve its address.
        const result = await sendTx(account, tx);
        const { contract } = result;

        const newContractAddress = contract.address.toString();
        log(getExplorerUrl(result));
        log(`New ${contractName} contract with address ${newContractAddress} is instantiated from existing wasm code with hash ${maybeCodeHash}`);

        registerABI(contractName, abi, maybeCodeHash);
        registerContract(contractName, chainName, newContractAddress);
    }
}

const getAbiPath = (contractName) => {
    contractName = contractName.toLowerCase();
    const abiPath = `./../target/ink/${contractName}/metadata.json`;
    return abiPath;
}

const getWasmPath = (contractName) => {
    contractName = contractName.toLowerCase();
    const wasmPath = `./../target/ink/${contractName}/${contractName}.wasm`;
    return wasmPath;
}


module.exports = {
    deployContract
};
