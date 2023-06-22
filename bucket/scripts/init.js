import * as fs from 'fs';

import { ContractPromise } from '@polkadot/api-contract';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { cryptoWaitReady, mnemonicGenerate } from "@polkadot/util-crypto";

const INIT_ENV = process.env.INIT_ENV;
const SUPERADMIN_MNEMONIC = process.env.SUPERADMIN;
const CERE = 10_000_000_000n;
const GEN = 0x10000000000000000n; // UINT64_MAX

const config = {
  devnet: {
    ws_provider: "wss://archive.devnet.cere.network/ws",
    contract_address: "6SfBsKbfPUTN35GCcqAHSMY4MemedK2A73VeJ34Z2FV6PB4r",
    cluster_param: { replicationFactor: 3 },
    vnodes: [ [0n], [GEN / 4n], [GEN * 2n / 4n], [GEN * 3n / 4n] ],
    storage_nodes: [1n, 2n, 3n, 4n],
    storage_node_params: [
      { url: `https://node-0.v2.storage.devnet.cere.network` },
      { url: `https://node-1.v2.storage.devnet.cere.network` },
      { url: `https://node-2.v2.storage.devnet.cere.network` },
      { url: `https://node-3.v2.storage.devnet.cere.network` },
    ],
    cdn_nodes: [1n, 2n, 3n, 4n],
    cdn_node_params: [
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
    ],
  },
  testnet: {
    ws_provider: "wss://archive.devnet.cere.network/ws",
    contract_address: "6R2PF5gzKYbNkNLymTr8YNeQgWqNkE6azspwaMLZF2UHc1sg",
    cluster_param: { replicationFactor: 3 },
    vnodes: [ [0n], [GEN / 3n], [GEN * 2n / 3n] ],
    storage_nodes: [1n, 2n, 3n],
    storage_node_params: [
      { url: `https://node-0.v2.us.storage.testnet.cere.network` },
      { url: `https://node-1.v2.us.storage.testnet.cere.network` },
      { url: `https://node-2.v2.us.storage.testnet.cere.network` },
    ],
    cdn_nodes: [1n, 2n],
    cdn_node_params: [
      {
        url: `https://node-0.v2.us.cdn.testnet.cere.network`,
        publicKey: "0x089522cee0567ff8e072c9efbd5cb4e05fe47cdab8340816be9d6f60538e8645",
      },
      {
        url: `https://node-1.v2.us.cdn.testnet.cere.network`,
        publicKey: "0x7693cbc6a6f3fff67d4eb29bb07bc018e1eee43618d03e6c0a91b0b3e79f272d",
      },
    ],
  },
};

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
  console.error("Please provide INIT_ENV");
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

console.log("6. cdnClusterCreate");
await signAndSendPromise(await contract.tx.cdnClusterCreate(txOptions, cfg.cdn_nodes), sadmin);

console.log("7. nodeCreate");
for (let i = 0; i < cfg.storage_node_params.length; i++) {
  const param = JSON.stringify(cfg.storage_node_params[i]);
  const user = createUser();
  fs.appendFileSync('secrets.txt', `${user.address}: ${user.mnemonic} -- ${INIT_ENV} storage ${i}\n`);
  console.log(`  node ${i}: address ${user.address}, param ${param}`);
  await signAndSendPromise(await contract.tx.nodeCreate(txOptions, 1n * CERE, param, 1000n, "ACTIVE", user.address), sadmin);
}

console.log("8. clusterCreate");
await signAndSendPromise(await contract.tx.clusterCreate(txOptions, alice.address, cfg.vnodes, cfg.storage_nodes, JSON.stringify(cfg.cluster_param)), sadmin);

console.log("9. clusterReserveResource");
await signAndSendPromise(await contract.tx.clusterReserveResource(txOptions, 1, 1000n), sadmin);

// console.log("cdnNodeChangeParams");
// for (let i = 0; i < cfg.cdn_node_params.length; i++) {
//   await signAndSendPromise(await contract.tx.cdnNodeChangeParams(txOptions, i+1, JSON.stringify(cfg.cdn_node_params[i])), sadmin);
// }

//console.log(res.events.map(event => event.event.toHuman()));
process.exit(0);
