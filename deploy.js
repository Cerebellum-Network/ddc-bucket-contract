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
const REUSE_CODE_HASH = "0x75f300198e48015c82edf1eade7f8eab198608a9cbe08a950920c3aa834d834d";
const REUSE_CONTRACT_ADDRESS = "5EjAx9r2VwMAJDg5zgbUCSfcZNKRz8p7ZoWGdsyjEkvnFYY8";

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
    const provider_id = account.address;
    const ownerId = account.address;
    const anyAccountId = account.address;
    const rent_per_month = 10n * CERE;
    const vnode_params = "{\"url\":\"https://ddc-123.cere.network/bucket/{BUCKET_ID}\"}";
    const bucket_params = "{}";

    let cluster_id;
    {
        log("Setup a cluster…");
        let cluster_params = "{}";
        const tx = contract.tx
            .clusterCreate(txOptions, cluster_params);

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
        cluster_id = ddcBucket.findCreatedClusterId(events);
        log("New Cluster", cluster_id);
    }

    let vnode_id;
    {
        log("Setup a vnode…");
        const tx = contract.tx
            .vnodeCreate(txOptions, cluster_id, rent_per_month, vnode_params);

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
        vnode_id = ddcBucket.findCreatedVNodeId(events);
        log("New VNode", vnode_id);
    }
    {
        log("\nRead vnode info…");
        const {result, output} = await contract.query
            .vnodeGet(anyAccountId, txOptions, vnode_id);

        if (!result.isOk) assert.fail(result.asErr);

        log('OUTPUT', output.toHuman());
        assert.deepEqual(output.toJSON(), {
            "ok": {
                vnode_id,
                provider_id,
                rent_per_month,
                vnode_params,
            },
        });
    }

    let bucketId;
    {
        log("Create a bucket…");
        const tx = contract.tx
            .bucketCreate(txOptions, bucket_params);

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
        bucketId = ddcBucket.findCreatedBucketId(events);
        log("New bucketId", bucketId);
    }
    let dealId;
    {
        log("Create a deal for the bucket…");
        const tx = contract.tx
            .bucketAllocIntoCluster(txOptionsPay, bucketId, cluster_id);

        const result = await sendTx(account, tx);
        const events = result.contractEvents || [];
        log(getExplorerUrl(result));
        log("EVENTS", JSON.stringify(events, null, 4));
        dealId = ddcBucket.findCreatedDealId(events);
        log("New dealId", dealId);
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
        log("\nRead deal status…");
        let {result, output} = await contract.query
            .dealGetStatus(anyAccountId, txOptions, dealId);

        if (!result.isOk) assert.fail(result.asErr);
        output = output.toJSON();
        log('OUTPUT', output);

        assert.deepEqual(output.ok.vnode_id, vnode_id);
        assert(output.ok.estimated_rent_end_ms > 0);
    }
    {
        log("\nRead bucket status…");
        let {result, output} = await contract.query
            .bucketGetStatus(anyAccountId, txOptions, bucketId);

        if (!result.isOk) assert.fail(result.asErr);
        output = output.toJSON();
        log('OUTPUT', output);

        /* TODO
        assert.deepEqual(output.ok.vnode_id, vnode_id);
        assert(output.ok.estimated_rent_end_ms > 0);
        assert.deepEqual(output.ok.writer_ids, [ownerId]);
        */
    }

    process.exit(0);
}

main().then(log, log);
