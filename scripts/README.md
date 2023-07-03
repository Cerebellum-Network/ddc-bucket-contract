## Pre requirements

In order to use the scripts described below, you need to [build DDC contracts](./../README.md), and install the [yarn](https://classic.yarnpkg.com/en/docs/install) package manager.
Also, check the [config file](./sdk/src/config/index.js), to ensure that you are running scripts with expected environment variables.


## Examples

The [examples](./examples/) folder contains scripts that demonstrate how to properly send transacrions to DDC Bucket contracts using the [@polkadot/api](https://polkadot.js.org/docs/api/) library. These simple scenarious should help other team members to clarify the business logic flow, and quickly detect issues related to various business constraints and infrastructue constraints (i.e. gas price, attached tokens, method signature, etc.). Scripts should be updated while the contract is evolving to reflect the actual logic.


#### DDC Bucket scenario

Run the script as:
```
ENV=devnet yarn run demo-ddc-bucket
```
The execution progress will be displayed in the console along with links to explorer that will help you to investigate the details of each transaction


#### Display DDC Bucket state

Run the script as:
```
ENV=devnet yarn run print-ddc-bucket
```


## Deployment using script

The [deployment](./deployment/) folder contains scripts that allow you to deploy artifacts to a local or remote network. 
Typically, these scripts are used for EDC, Devnet and QAnet environments during development. 
For Testnet and Mainnet environments, it is recomended to upload the contract code manually and assert all the required keys, attached tokens, gas limits, etc. during deployment and contract instantianating.


#### DDC Bucket deployment

Run the script as:
```
yarn run deploy-ddc-bucket
```
Optionally, the command can accept a code hash as the first parameter, and constructor name as the second parameter. In order to use these options, your contract artifacts [must be registered](./sdk/src/deploymentRegistry.js) to retrieve the required metadata from artifacts.


#### Build and Deploy all contracts

To run both building and deployment for all contracts, you can use the [build-and-deploy.sh](./../build-and-deploy.sh) script.
