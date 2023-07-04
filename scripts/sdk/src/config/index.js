// todo: introduce DOTENV and allow reading variables from environment

const ACTOR_SEED = "//Alice";

const EXPLORER_URL = "https://explorer.cere.network";
const DDC_BUCKET_CONTRACT_NAME = "ddc_bucket";

const DEVNET_RPC_ENDPOINT = "wss://archive.devnet.cere.network/ws/";
const DEVNET_DDC_BUCKET_ADDR = "";
const DEVNET_CHAIN_NAME = "Cere Devnet";

const TESTNET_RPC_ENDPOINT = "wss://archive.testnet.cere.network/ws/";
const TESTNET_DDC_BUCKET_ADDR = "";
const TESTNET_CHAIN_NAME = "Cere Testnet";

const LOCAL_RPC_ENDPOINT = "ws://127.0.0.1:9944/";
const LOCAL_DDC_BUCKET_ADDR = ""; // add your local address
const LOCAL_CHAIN_NAME = "Development";

module.exports = {
  ACTOR_SEED,
  EXPLORER_URL,

  DDC_BUCKET_CONTRACT_NAME,

  DEVNET_RPC_ENDPOINT,
  DEVNET_DDC_BUCKET_ADDR,
  DEVNET_CHAIN_NAME,

  TESTNET_RPC_ENDPOINT,
  TESTNET_DDC_BUCKET_ADDR,
  TESTNET_CHAIN_NAME,

  LOCAL_RPC_ENDPOINT,
  LOCAL_DDC_BUCKET_ADDR,
  LOCAL_CHAIN_NAME
}