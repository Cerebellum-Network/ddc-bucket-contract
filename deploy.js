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
const REUSE_CODE_HASH = "0xc9e137e30ece8a464286e92cea70e4181ce3a80a2c4602da768e53632c5027d6";
const REUSE_CONTRACT_ADDRESS = "5CNZigA12EL2LvE4DmQ5mNppjpJKumCTBx8suzZQEdL9rHQz";

const WASM = `./target/ink/${CONTRACT_NAME}/${CONTRACT_NAME}.wasm`;
const ABI = `./target/ink/${CONTRACT_NAME}/metadata.json`;
const CONSTRUCTOR = "new";

const SEED = "//Alice";
const RPC = "wss://rpc.testnet.cere.network:9945";


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
        value: 0n,
        gasLimit: -1, //100_000n * MGAS,
    };
    const txOptionsPay = {
        value: 1n * CERE,
        gasLimit: -1, //100_000n * MGAS,
    };

    // Test data.
    const providerId = account.address;
    const ownerId = account.address;
    const anyAccountId = account.address;
    const rent_per_month = 10n * CERE;
    const location = "https://ddc.dev.cere.network/bucket/{BUCKET_ID}";

    {
        log("Setup a provider…");
        const tx = contract.tx
            .providerSetInfo(txOptions, rent_per_month, location);

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
    }
    {
        log("\nRead provider info…");
        const {result, output} = await contract.query
            .providerGetInfo(anyAccountId, txOptions, providerId);

        if (!result.isOk) assert.fail(result.asErr);

        log('OUTPUT', output.toHuman());
        assert.deepEqual(output.toJSON(), {
            "ok": {
                rent_per_month,
                location,
            },
        });
    }

    let bucketId;
    {
        log("Create a bucket…");
        const tx = contract.tx
            .bucketCreate(txOptionsPay, providerId);

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
        bucketId = ddcBucket.findCreatedBucketId(events);
        log("New bucketId", bucketId);
    }
    {
        log("Topup the account…");
        const tx = contract.tx
            .deposit(txOptionsPay);

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
    }
    {
        log("\nRead bucket status…");
        let {result, output} = await contract.query
            .bucketGetStatus(anyAccountId, txOptions, bucketId);

        if (!result.isOk) assert.fail(result.asErr);
        output = output.toJSON();
        log('OUTPUT', output);

        assert.equal(output.ok.provider_id, providerId);
        assert(output.ok.estimated_rent_end_ms > 0);
        assert.deepEqual(output.ok.writer_ids, [ownerId]);
    }

    process.exit(0);
}

main().then(log, log);
