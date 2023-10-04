# Common AMM

This repository contains implementations of AMM DEXes written for Common product.

There will be multiple AMM models implemented, each being the most suitable model for a certain token pair (stablecoin pairs being created in a CFM based on Curve StableSwap, PSP22 token pair on UniswapV2, etc.)

Currently, this repository contains the line by line implementation of [uniswap-v2 core](https://github.com/Uniswap/v2-core) and [uniswap-v2 periphery](https://github.com/Uniswap/v2-periphery). Code was adapted to match Subatrate platform and ink! language.

### Purpose

This is an unaudited full dex implementation ready to be used.

### Versions

[ink! 4.3.0](https://github.com/paritytech/ink/tree/v4.3.0)

### License

Apache 2.0

## 🏗️ How to use - Contracts

##### 💫 Build

Use these [instructions](https://use.ink/getting-started/setup) to set up your ink!/Rust environment.
To build all contracts, run this command from the project root directory:

```sh
make build-all
```

##### 💫 Wrap

Use these [instructions](https://github.com/Cardinal-Cryptography/ink-wrapper#installation) to set up your `ink-wrapper` environment.
Once you have built your contracts, you can wrap them by running this command from the project root directory:

```sh
make wrap-all
```

You can also build and wrap the contracts in one step using:

```sh
make build-and-wrap-all
```

##### 💫 Run checks

Rust code checks and unit tests can be run from the root directory of the project:

```sh
make check-all
```

##### 💫 Run unit test

To manually run unit tests, use:

```sh
cargo test
```

##### 💫 Run E2E tests

To run the E2E test suite, execute the following command from the root directory of the project.

```sh
make e2e-tests-with-setup-and-teardown
```

This will:

- Build and wrap your contracts.
- Run a single node.
- Sequentially run all the E2E test cases with setup.
- Stop the node.

##### 💫 Deploy

First start your local node. You can do that by running `make start-node` in the root directory of the project.

To deploy contracts, execute `npm run deploy:local` in the root directory.

To create sample tokens and register them as pairs in the DEX, run `npm run example:local`.

Note that this requires rebuilding TypeScript wrappers first: `npm run compile:release`.

##### 💫 Help

You can see a list of available `make` recipes by running:

```sh
make help
```
