const {ContractPromise} = require('@polkadot/api-contract');
const {getABI} = require("./abiRegistry.js");


const ADDRESSES = {};

function registerContract(name, environment, address) {
    ADDRESSES[`${name}@${environment}`] = address;
}

function getContract(name, environment, api) {
    const abi = getABI(name);
    if (!abi) return null;

    const address = ADDRESSES[`${name}@${environment}`];
    if (!address) return null;

    return new ContractPromise(api, abi, address);
}


module.exports = {
    registerContract,
    getContract,
};
