const {ApiPromise, WsProvider} = require('@polkadot/api');
const {Keyring} = require('@polkadot/api');
const cereTypes = require("./cere_custom_types.json");
const config = require("./config");
const { mnemonicGenerate } = require('@polkadot/util-crypto');

async function connect(rpcUrl) {
    const wsProvider = new WsProvider(rpcUrl);
    const api = await ApiPromise.create({
        provider: wsProvider,
        types: cereTypes,
    });
    const chainName = api.runtimeChain.toString();

    const getExplorerUrl = (result) => {
        const blockHash = result.status.asInBlock.toString();
        const blockUrl = `${config.EXPLORER_URL}/?rpc=${rpcUrl}#/explorer/query/${blockHash}`;
        return blockUrl;
    };

    return {api, chainName, getExplorerUrl};
}

function randomAccount() {
    const keyring = new Keyring({type: 'sr25519'});
    const mnemonic = mnemonicGenerate(12);
    const account = keyring.addFromMnemonic(mnemonic);
  
    return {
        account,
        mnemonic,
        address: account.address,
    };
}

function accountFromUri(uri) {
    const keyring = new Keyring({type: 'sr25519'});
    const account = keyring.addFromUri(uri);
    return account;
}

async function sendTx(account, tx) {
    const result = await new Promise(async (resolve, reject) => {
        const unsub = await tx.signAndSend(account, (result) => {
            if (result.status.isInBlock || result.status.isFinalized) {
                unsub();
                resolve(result);
            }
        });
    });
    return result;
}


module.exports = {
    connect,
    accountFromUri,
    sendTx,
    randomAccount,
};
