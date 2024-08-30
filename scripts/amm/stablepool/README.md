## Prerequisites

The scripts use typechain-generated types for contract deployment and interaction. To generate the types:

1. Build the contracts by executing `make build-all` or `make build-dockerized` in the root directory.
2. Generate typechain types by executing `npm i && npm run compile` in the root directory.

## Stablepool deployment

1. Create `.env` file as shown in `.env.example`.
2. Create `deploymentPoolsParams.json` and add deployment parameters as shown in `deploymentPoolsParams.example.json`.
3. Execute `npm run deploy-stable` in the root directory.

To test deployment script locally with the `*.example` files, start a local node with `make start-node` and execute `npm run deploy-stable example` in the root directory.
