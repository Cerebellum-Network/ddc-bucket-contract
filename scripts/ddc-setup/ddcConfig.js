const GEN = 0x10000000000000000n; // UINT64_MAX

module.exports = {
  devnet: {
    blockchainUrl: "wss://archive.devnet.cere.network/ws",
    clusters: [
      {
        params: { replicationFactor: 3 },
        storageNodes: [
          {
            pubKey: "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
            vNodes: [0n],
            params: { url: `https://node-0.v2.storage.devnet.cere.network` },
          },
          {
            pubKey: "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48",
            vNodes: [GEN / 4n],
            params: { url: `https://node-1.v2.storage.devnet.cere.network` },
          },
          {
            pubKey: "0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22",
            vNodes: [GEN * 2n / 4n],
            params: { url: `https://node-2.v2.storage.devnet.cere.network` },
          },
          {
            pubKey: "0x306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20",
            vNodes: [GEN * 3n / 4n],
            params: { url: `https://node-3.v2.storage.devnet.cere.network` },
          }
        ],
        cdnNodes: [
          {
            pubKey: "0xfe65717dad0447d715f660a0a58411de509b42e6efb8375f562f58a554d5860e",
            params: { url: `https://node-0.v2.cdn.devnet.cere.network` },
          },
          {
            pubKey: "0x1e07379407fecc4b89eb7dbd287c2c781cfb1907a96947a3eb18e4f8e7198625",
            params: { url: `https://node-1.v2.cdn.devnet.cere.network` },
          },
          {
            pubKey: "0xe860f1b1c7227f7c22602f53f15af80747814dffd839719731ee3bba6edc126c",
            params: { url: `https://node-2.v2.cdn.devnet.cere.network` },
          },
          {
            pubKey: "0x8ac59e11963af19174d0b94d5d78041c233f55d2e19324665bafdfb62925af2d",
            params: { url: `https://node-3.v2.cdn.devnet.cere.network` },
          }
        ]
      },
    ],
  },
  testnet: {
    blockchainUrl: "wss://archive.testnet.cere.network/ws",
    clusters: [
      {
        params: { replicationFactor: 3 },
        storageNodes: [
          {
            pubKey: "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
            vNodes: [0n],
            params: { url: `https://node-0.v2.us.storage.testnet.cere.network` },
          },
          {
            pubKey: "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48",
            vNodes: [GEN / 3n],
            params: { url: `https://node-1.v2.us.storage.testnet.cere.network` },
          },
          {
            pubKey: "0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22",
            vNodes: [GEN * 2n / 3n],
            params: { url: `https://node-2.v2.us.storage.testnet.cere.network` },
          }
        ],
        cdnNodes: [
          {
            pubKey: "0xfe65717dad0447d715f660a0a58411de509b42e6efb8375f562f58a554d5860e",
            params: { url: `https://node-0.v2.us.cdn.testnet.cere.network` },
          },
          {
            pubKey: "0x1e07379407fecc4b89eb7dbd287c2c781cfb1907a96947a3eb18e4f8e7198625",
            params: { url: `https://node-1.v2.us.cdn.testnet.cere.network` },
          }
        ]
      },
      {
        params: { replicationFactor: 3 },
        storageNodes: [
          {
            pubKey: "0x306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20",
            vNodes: [0n],
            params: { url: `https://node-0.v2.eu.storage.testnet.cere.network` },
          },
          {
            pubKey: "0xe659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e",
            vNodes: [GEN / 3n],
            params: { url: `https://node-1.v2.eu.storage.testnet.cere.network` },
          },
          {
            pubKey: "0x1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c",
            vNodes: [GEN * 2n / 3n],
            params: { url: `https://node-2.v2.eu.storage.testnet.cere.network` },
          }
        ],
        cdnNodes: [
          {
            pubKey: "0xe860f1b1c7227f7c22602f53f15af80747814dffd839719731ee3bba6edc126c",
            params: { url: `https://node-0.v2.eu.cdn.testnet.cere.network` },
          },
          {
            pubKey: "0x8ac59e11963af19174d0b94d5d78041c233f55d2e19324665bafdfb62925af2d",
            params: { url: `https://node-1.v2.eu.cdn.testnet.cere.network` },
          }
        ]
      }
    ],
  },

  // mainnet: {
  //   ws_provider: "wss://archive.testnet.cere.network/ws",
  //   contract_address: "6So8eqxMyWAxJ4ZZ2wCcJym7Cy6BYkc4V8GZZD9wgdCqWMQB",
  //   cluster: {
  //     1n: {
  //       param: { replicationFactor: 3 },
  //       vnodes: [ [0n], [GEN / 3n], [GEN * 2n / 3n] ],
  //       storage_nodes: [1n, 2n, 3n],
  //     },
  //   },
  //   storage_node_params: [
  //     { url: `https://node-0.v2.us.storage.mainnet.cere.network` },
  //     { url: `https://node-1.v2.us.storage.mainnet.cere.network` },
  //     { url: `https://node-2.v2.us.storage.mainnet.cere.network` },
  //   ],
  //   cdn_cluster: {
  //     0n: {
  //       cdn_nodes: [1n, 2n],
  //     },
  //   },
  //   cdn_node_params: [
  //     {
  //       url: `https://node-0.v2.us.cdn.mainnet.cere.network`,
  //       publicKey: "0x86af4db1e433ad221b6fa3c1a9fc4de694ab59408ca57584e50d8fd420e7b45b",
  //     },
  //     {
  //       url: `https://node-1.v2.us.cdn.mainnet.cere.network`,
  //       publicKey: "0x9a9fb6c479ef7c8f3af54dc0720f08a73d532815d525aa8d69d965e56512440e",
  //     },
  //   ],
  // },


};
