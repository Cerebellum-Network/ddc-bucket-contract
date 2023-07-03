const GEN = 0x10000000000000000n; // UINT64_MAX

module.exports = {
  devnet: {
    ws_provider: "wss://archive.devnet.cere.network/ws",
    contract_address: "6SfBsKbfPUTN35GCcqAHSMY4MemedK2A73VeJ34Z2FV6PB4r",
    cluster: {
      1n: {
        param: { replicationFactor: 3 },
        vnodes: [ [0n], [GEN / 4n], [GEN * 2n / 4n], [GEN * 3n / 4n] ],
        storage_nodes: [1n, 2n, 3n, 4n],
      },
    },
    storage_node_params: [
      { url: `https://node-0.v2.storage.devnet.cere.network` },
      { url: `https://node-1.v2.storage.devnet.cere.network` },
      { url: `https://node-2.v2.storage.devnet.cere.network` },
      { url: `https://node-3.v2.storage.devnet.cere.network` },
    ],
    cdn_cluster: {
      0n: {
        cdn_nodes: [1n, 2n, 3n, 4n],
      },
    },
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
    // contract_address: "6R2PF5gzKYbNkNLymTr8YNeQgWqNkE6azspwaMLZF2UHc1sg",
    contract_address: "6UWDf6rEgSDFRr1h2pMdCbifowTDK64yRkDR6nc3C1cjL82e",
    cluster: {
      1n: {
        storage_nodes: [1n, 2n, 3n],
        vnodes: [ [0n], [GEN / 3n], [GEN * 2n / 3n] ],
        param: { replicationFactor: 3 },
      },
      2n: {
        storage_nodes: [4n, 5n, 6n],
        vnodes: [ [0n], [GEN / 3n], [GEN * 2n / 3n] ],
        param: { replicationFactor: 3 },
      },
    },
    storage_node_params: [
      { url: `https://node-0.v2.us.storage.testnet.cere.network` },
      { url: `https://node-1.v2.us.storage.testnet.cere.network` },
      { url: `https://node-2.v2.us.storage.testnet.cere.network` },
      { url: `https://node-0.v2.eu.storage.testnet.cere.network` },
      { url: `https://node-1.v2.eu.storage.testnet.cere.network` },
      { url: `https://node-2.v2.eu.storage.testnet.cere.network` },
    ],
    cdn_cluster: {
      0n: {
        cdn_nodes: [1n, 2n],
      },
      1n: {
        cdn_nodes: [3n, 4n],
      },
      2n: {
        cdn_nodes: [5n, 6n],
      },
    },
    cdn_node_params: [
      {
        url: `https://node-0.v2.us.cdn.testnet.cere.network`,
        publicKey: "0x089522cee0567ff8e072c9efbd5cb4e05fe47cdab8340816be9d6f60538e8645",
      },
      {
        url: `https://node-1.v2.us.cdn.testnet.cere.network`,
        publicKey: "0x7693cbc6a6f3fff67d4eb29bb07bc018e1eee43618d03e6c0a91b0b3e79f272d",
      },
      {
        url: `https://node-0.v2.eu.cdn.testnet.cere.network`,
        publicKey: "0xdce47cdd1da69c19261b72e3c58e93d78e49d1ac20a566b535cb9bcf9d197958",
      },
      {
        url: `https://node-1.v2.eu.cdn.testnet.cere.network`,
        publicKey: "0xb8541743735ffba6877b214925a9ec07c813369bb36b49ee5849b1fea0f9dd55",
      },
      {
        url: `https://node-0.unmarshal.v2.us.cdn.testnet.cere.network`,
        publicKey: "0x685168b78deb42eebf01e38d18c9302f032c50e544d56c1c4f86b13b0a2ad40a",
      },
      {
        url: `https://node-1.unmarshal.v2.us.cdn.testnet.cere.network`,
        publicKey: "0xeeb3683dcd43e9c7f8759b1dce2440d767ae1c51dec05b584d785e24997cb947",
      },
    ],
  },

  mainnet: {
    ws_provider: "wss://archive.testnet.cere.network/ws",
    contract_address: "6So8eqxMyWAxJ4ZZ2wCcJym7Cy6BYkc4V8GZZD9wgdCqWMQB",
    cluster: {
      1n: {
        param: { replicationFactor: 3 },
        vnodes: [ [0n], [GEN / 3n], [GEN * 2n / 3n] ],
        storage_nodes: [1n, 2n, 3n],
      },
    },
    storage_node_params: [
      { url: `https://node-0.v2.us.storage.mainnet.cere.network` },
      { url: `https://node-1.v2.us.storage.mainnet.cere.network` },
      { url: `https://node-2.v2.us.storage.mainnet.cere.network` },
    ],
    cdn_cluster: {
      0n: {
        cdn_nodes: [1n, 2n],
      },
    },
    cdn_node_params: [
      {
        url: `https://node-0.v2.us.cdn.mainnet.cere.network`,
        publicKey: "0x86af4db1e433ad221b6fa3c1a9fc4de694ab59408ca57584e50d8fd420e7b45b",
      },
      {
        url: `https://node-1.v2.us.cdn.mainnet.cere.network`,
        publicKey: "0x9a9fb6c479ef7c8f3af54dc0720f08a73d532815d525aa8d69d965e56512440e",
      },
    ],
  },
};
