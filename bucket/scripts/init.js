import * as fs from 'fs';

import { ContractPromise } from '@polkadot/api-contract';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { cryptoWaitReady, mnemonicGenerate } from "@polkadot/util-crypto";

const WS_PROVIDER = "wss://archive.devnet.cere.network/ws";
const ADDRESS = "6SfBsKbfPUTN35GCcqAHSMY4MemedK2A73VeJ34Z2FV6PB4r";
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

console.log(await contract.query.getFeeBp(alice.address, txOptions));

console.log("1. adminGrantPermission");
const res = await signAndSendPromise(await contract.tx.adminGrantPermission(txOptions, sadmin.address, "SuperAdmin"), sadmin);

console.log("2. accountSetUsdPerCere");
const res = await signAndSendPromise(await contract.tx.accountSetUsdPerCere(txOptions, 1000n), sadmin);

console.log("3. cdnNodeTrustManager");
const res = await signAndSendPromise(await contract.tx.cdnNodeTrustManager(txOptions, sadmin.address), sadmin);

console.log("4. nodeTrustManager");
const res = await signAndSendPromise(await contract.tx.nodeTrustManager(txOptions, sadmin.address), sadmin);

console.log("5. cdnNodeCreate");
const res1 = await signAndSendPromise(await contract.tx.cdnNodeCreate(txOptions, JSON.stringify({ url: `https://node-0.v2.cdn.devnet.cere.network` })), sadmin);
const res2 = await signAndSendPromise(await contract.tx.cdnNodeCreate(txOptions, JSON.stringify({ url: `https://node-1.v2.cdn.devnet.cere.network` })), sadmin);
const res3 = await signAndSendPromise(await contract.tx.cdnNodeCreate(txOptions, JSON.stringify({ url: `https://node-2.v2.cdn.devnet.cere.network` })), sadmin);
const res4 = await signAndSendPromise(await contract.tx.cdnNodeCreate(txOptions, JSON.stringify({ url: `https://node-3.v2.cdn.devnet.cere.network` })), sadmin);

console.log("6. cdnClusterCreate");
const res = await signAndSendPromise(await contract.tx.cdnClusterCreate(txOptions, [1n, 2n, 3n, 4n]), sadmin);

console.log("6. nodeCreate");
for (let i = 0; i < 4; i++) {
  const params = JSON.stringify({ url: `https://node-${i}.v2.storage.devnet.cere.network` });
  const user = createUser();
  fs.appendFileSync('secrets.txt', `${user.address}: ${user.mnemonic} -- storage ${i}\n`);
  console.log(`  node ${i}: address ${user.address}, param ${params}`); 
  await signAndSendPromise(await contract.tx.nodeCreate(txOptions, 1n * CERE, params, 100000, "ACTIVE", user.address), sadmin);
}

console.log(res.events.map(event => event.event.toHuman()));
process.exit(0);


