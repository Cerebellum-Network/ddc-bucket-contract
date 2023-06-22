import * as fs from 'fs';

import { ContractPromise } from '@polkadot/api-contract';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { cryptoWaitReady, mnemonicGenerate } from "@polkadot/util-crypto";

const WS_PROVIDER = "wss://archive.devnet.cere.network/ws";
const ADDRESS = "6SfBsKbfPUTN35GCcqAHSMY4MemedK2A73VeJ34Z2FV6PB4r";
const CERE = 10_000_000_000n;

const cdnNodeParams = [
  {
    url: `https://node-0.v2.cdn.devnet.cere.network`,
    publicKey: "0x1c4a1b081af8dd09096ebb6e7ad61dd549ac2931cdb2b1216589094ad71ca90b",
  },
  {
    url: `https://node-1.v2.cdn.devnet.cere.network`,
    publicKey: "0x3ec2ec407053acdfe8137d7105e90294f2e0e5f5fe5420fd3172142671dbc25f",
  },
  {
    url: `https://node-2.v2.cdn.devnet.cere.network`,
    publicKey: "0x20e448c403d3f009ec309394d3aab828c3dbf0d2cc8047f01dded984ec992b41",
  },
  {
    url: `https://node-3.v2.cdn.devnet.cere.network`,
    publicKey: "0xd2f93cea79e37cfc9e5f78cd3e51b989afb1e257adcbbae00b8cd081539e9f13",
  }
];

const txOptions = {
  storageDepositLimit: null,
  gasLimit: 100_000_000_000n,
};

async function signAndSendPromise(txn, signer) {
  return new Promise((res, rej) => {
    txn
      .signAndSend(signer, ({ events = [], status, blockNumber }) => {
        if (status.isInvalid) {
          console.log("    Transaction invalid");
          rej("Transaction invalid");
        } else if (status.isReady) {
          console.log("    Transaction is ready");
        } else if (status.isBroadcast) {
          console.log("    Transaction has been broadcasted");
        } else if (status.isInBlock) {
          const blockHash = status.asInBlock.toHex();
          console.log(`    Transaction is in block: ${blockHash} of ${blockNumber}`);
        } else if (status.isFinalized) {
          const blockHash = status.asFinalized.toHex();
          console.log(`    Transaction has been included in blockHash ${blockHash} of ${blockNumber}`);
          const treasuryDeposit = events.find(
            (event) => event.event.toHuman().method === "Deposit" && event.event.toHuman().section === "treasury",
          );
          const txFee = treasuryDeposit ? treasuryDeposit.event.toHuman().data.value : undefined;
          const txFeeParsed = txFee ? (parseFloat(txFee.replace(" mCERE", "")) / 1000) * 2 : undefined;

          if (events.find(event => event.event.toHuman().method === "ExtrinsicSuccess")) res({ blockHash, txFeeParsed, events });
          else rej("No success found: " + blockHash);
        }
      })
      .catch((err) => rej(err));
  });
}

function createUser() {
  const keyring = new Keyring({ type: "sr25519" });
  const mnemonic = mnemonicGenerate(12);
  const account = keyring.addFromUri(mnemonic);

  return {
    mnemonic: mnemonic,
    address: account.address,
    addressBase64: Buffer.from(account.publicKey).toString("base64"),
  };
}

await cryptoWaitReady();
const keyring = new Keyring({ type: "sr25519" });
const alice = keyring.addFromUri("//Alice");
const sadmin = keyring.addFromUri(process.env.SUPERADMIN);

console.log(`Superadmin: ${sadmin.address}`);

// Contract metadata
const metadata = fs.readFileSync('./metadata.json', 'utf8');

// Construct
const wsProvider = new WsProvider(WS_PROVIDER);
const api = await ApiPromise.create({ provider: wsProvider });
const contract = new ContractPromise(api, metadata, ADDRESS);

// const result = await contract.query.cdnNodeList(alice.address, txOptions, 0, 100, null);
// console.log(result.output);

console.log("1. adminGrantPermission");
const res = await signAndSendPromise(await contract.tx.adminGrantPermission(txOptions, sadmin.address, "SuperAdmin"), sadmin);

console.log("2. accountSetUsdPerCere");
const res = await signAndSendPromise(await contract.tx.accountSetUsdPerCere(txOptions, 1000n), sadmin);

console.log("3. cdnNodeTrustManager");
const res = await signAndSendPromise(await contract.tx.cdnNodeTrustManager(txOptions, sadmin.address), sadmin);

console.log("4. nodeTrustManager");
const res = await signAndSendPromise(await contract.tx.nodeTrustManager(txOptions, sadmin.address), sadmin);

console.log("5. cdnNodeCreate");
for (let i = 0; i < cdnNodeParams.length; i++) {
  await signAndSendPromise(await contract.tx.cdnNodeCreate(txOptions, JSON.stringify(cdnNodeChangeParamss[i]), sadmin);
}

console.log("6. cdnClusterCreate");
const res = await signAndSendPromise(await contract.tx.cdnClusterCreate(txOptions, [1n, 2n, 3n, 4n]), sadmin);

console.log("7. nodeCreate");
for (let i = 0; i < 4; i++) {
  const params = JSON.stringify({ url: `https://node-${i}.v2.storage.devnet.cere.network` });
  const user = createUser();
  fs.appendFileSync('secrets.txt', `${user.address}: ${user.mnemonic} -- storage ${i}\n`);
  console.log(`  node ${i}: address ${user.address}, param ${params}`); 
  await signAndSendPromise(await contract.tx.nodeCreate(txOptions, 1n * CERE, params, 100000, "ACTIVE", user.address), sadmin);
}

console.log("8. clusterCreate");
const cparams = JSON.stringify({ replicationFactor: 3 });
const gen = 0x10000000000000000n / 4n;
const vNodes = [ [0n], [gen], [gen*2n], [gen*3n] ];
const sNodes = [1n, 2n, 3n, 4n];
console.log(cparams, sNodes, vNodes);
const res = await signAndSendPromise(await contract.tx.clusterCreate(txOptions, alice.address, vNodes, sNodes, cparams), sadmin);

console.log("9. clusterReserveResource");
const res = await signAndSendPromise(await contract.tx.clusterReserveResource(txOptions, 1, "100000"), sadmin);

// console.log("cdnNodeChangeParams");
// for (let i = 0; i < cdnNodeParams.length; i++) {
//   await signAndSendPromise(await contract.tx.cdnNodeChangeParams(txOptions, i+1, JSON.stringify(cdnNodeParams[i])), sadmin);
// }

//console.log(res.events.map(event => event.event.toHuman()));
process.exit(0);


