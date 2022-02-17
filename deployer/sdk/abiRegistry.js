const {CodePromise, BlueprintPromise} = require('@polkadot/api-contract');


const ABIS = {};
const CODE_HASHES = {};

function registerABI(name, abi, codeHash /*optional*/) {
    ABIS[name] = abi;
    CODE_HASHES[name] = codeHash;
}

function getABI(name) {
    return ABIS[name] || null;
}

function getBlueprint(name, api) {
    const abi = ABIS[name];
    const codeHash = CODE_HASHES[name];
    if (!(abi && codeHash)) return null;
    return new BlueprintPromise(api, abi, codeHash);
}

function getCodeDeployer(api, abi, wasm) {
    return new CodePromise(api, abi, wasm);
}


module.exports = {
    registerABI,
    getABI,
    getBlueprint,
    getCodeDeployer,
};
