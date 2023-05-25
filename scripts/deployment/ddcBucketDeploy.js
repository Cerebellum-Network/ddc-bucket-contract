const { deployContract } = require("./deploy");
const { config } = require("./../sdk");
const log = console.log;


async function main() {
    const args = process.argv.slice(2);
    const codeHash =  args.length > 0 ? args[0] : null;
    const constructorName = args.length > 1 ? args[1] : "new";

    await deployContract(config.DDC_BUCKET_CONTRACT_NAME, codeHash, constructorName);
    process.exit(0);
}

main().then(log, log);
