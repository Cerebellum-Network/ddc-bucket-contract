const { deployContract } = require("./deploy");
const { config } = require("./../sdk");
const log = console.log;

const CODE_HASH = process.env.CODE_HASH || null;

async function main() {
    await deployContract(config.DDC_BUCKET_CONTRACT_NAME, CODE_HASH);
    process.exit(0);
}

main().then(log, log);
