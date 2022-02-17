const {ContractPromise} = require('@polkadot/api-contract');
const {getABI} = require("./abiRegistry.js");


const ADDRESSES = {};

function registerContract(environment, name, address) {
    ADDRESSES[`${environment}/${name}`] = address;
}

function getContract(environment, name, api) {
    const abi = getABI(name);
    if (!abi) return null;

    const address = ADDRESSES[`${environment}/${name}`];
    if (!address) return null;

    return new ContractPromise(api, abi, address);
}


module.exports = {
    registerContract,
    getContract,
};
