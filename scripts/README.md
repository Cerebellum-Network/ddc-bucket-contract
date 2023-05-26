## Examples

The [examples](./examples/) folder contains scripts that demonstrates how to properly send transacrions to DDC Bucket and DDC Registry contracts using the [@polkadot/api](https://polkadot.js.org/docs/api/) library. These simple scenarious should help other team members to clarify the business logic flow, and quickly detect issues related to various business constraints and infrastructue constraints (i.e. gas price, attached tokens, method signature, etc.). Scripts should be updated while the contract is evolving to reflect the actual logic.

### Pre requirements

Prior to running, you need to [build DDC contracts](./../README.md), and install the [yarn](https://classic.yarnpkg.com/en/docs/install) package manager.
Also, check the [config file](./sdk/src/config/index.js), to ensure that you are running the script with expected environment variables.


#### DDC Bucket scenario

Run the script as:
```
yarn run demo-ddc-bucket
```
The execution progress will be displayed in the console along with links to explorer that will help you to investigate the details of each transaction


#### DDC NFT Registry scenario

Run the script as:
```
yarn run demo-nft-registry
```

The execution progress will be displayed in the console along with links to explorer that will help you to investigate the details of each transaction


#### Display DDC Bucket state

Run the script as:
```
yarn run print-ddc-bucket
```
