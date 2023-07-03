import { config } from './config.js';
import * as fs from 'fs';

import { ContractPromise } from '@polkadot/api-contract';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { cryptoWaitReady, mnemonicGenerate } from "@polkadot/util-crypto";

const INIT_ENV = process.env.INIT_ENV;
const SUPERADMIN_MNEMONIC = process.env.SUPERADMIN;
const CERE = 10_000_000_000n;

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

const cfg = config[INIT_ENV];
if (cfg === undefined) {
  console.error("Please provide INIT_ENV as one of ", Object.keys(config));
  process.exit(-1);
}
console.log(cfg);

await cryptoWaitReady();
const keyring = new Keyring({ type: "sr25519" });
const alice = keyring.addFromUri("//Alice");
const sadmin = keyring.addFromUri(SUPERADMIN_MNEMONIC);
console.log(`Superadmin: ${sadmin.address}`);

// Contract metadata
const metadata = fs.readFileSync('./metadata.json', 'utf8');

// Construct
const wsProvider = new WsProvider(cfg.ws_provider);
const api = await ApiPromise.create({ provider: wsProvider });
const contract = new ContractPromise(api, metadata, cfg.contract_address);

console.log("1. adminGrantPermission");
const res = await signAndSendPromise(await contract.tx.adminGrantPermission(txOptions, sadmin.address, "SuperAdmin"), sadmin);

console.log("2. accountSetUsdPerCere");
await signAndSendPromise(await contract.tx.accountSetUsdPerCere(txOptions, 1000n * CERE), sadmin);

console.log("3. cdnNodeTrustManager");
await signAndSendPromise(await contract.tx.cdnNodeTrustManager(txOptions, sadmin.address), sadmin);

console.log("4. nodeTrustManager");
await signAndSendPromise(await contract.tx.nodeTrustManager(txOptions, sadmin.address), sadmin);

console.log("5. cdnNodeCreate");
for (let i = 0; i < cfg.cdn_node_params.length; i++) {
  await signAndSendPromise(await contract.tx.cdnNodeCreate(txOptions, JSON.stringify(cfg.cdn_node_params[i])), sadmin);
}

for (let id in cfg.cdn_cluster) {
  const clu = cfg.cdn_cluster[id];
  console.log("6. cdnClusterCreate, cluster: ", id, clu);
  await signAndSendPromise(await contract.tx.cdnClusterCreate(txOptions, clu.cdn_nodes), sadmin);
}

console.log("7. nodeCreate");
for (let i = 0; i < cfg.storage_node_params.length; i++) {
  const param = JSON.stringify(cfg.storage_node_params[i]);
  const user = createUser();
  fs.appendFileSync('secrets.txt', `${user.address}: ${user.mnemonic} -- ${INIT_ENV} storage ${i}\n`);
  console.log(`  node ${i}: address ${user.address}, param ${param}`);
  await signAndSendPromise(await contract.tx.nodeCreate(txOptions, 1n * CERE, param, 100000n, "ACTIVE", user.address), sadmin);
}

for (let id in cfg.cluster) {
  const clu = cfg.cluster[id];

  console.log("8. clusterCreate, cluster: ", id, clu);
  await signAndSendPromise(await contract.tx.clusterCreate(txOptions, alice.address, clu.vnodes, clu.storage_nodes, JSON.stringify(clu.param)), sadmin);

  console.log("9. clusterReserveResource, cluster: ", id);
  await signAndSendPromise(await contract.tx.clusterReserveResource(txOptions, id, 100000n), sadmin);
}

// console.log("cdnNodeChangeParams");
// for (let i = 0; i < cfg.cdn_node_params.length; i++) {
//   await signAndSendPromise(await contract.tx.cdnNodeChangeParams(txOptions, i+1, JSON.stringify(cfg.cdn_node_params[i])), sadmin);
// }

//console.log(res.events.map(event => event.event.toHuman()));
process.exit(0);
