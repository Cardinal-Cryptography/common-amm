# DEX - UniswapV2
This folder contains the line by line implementation of [uniswap-v2 core](https://github.com/Uniswap/v2-core) and [uniswap-v2 periphery](https://github.com/Uniswap/v2-periphery) with its tests.

### Purpose
This is an unaudited full dex implementation ready to be used.

### Versions
[ink! 4.0.0](https://github.com/paritytech/ink/tree/v4.0.0)   
[openbrush 3.0.0](https://github.com/727-Ventures/openbrush-contracts/tree/3.0.0)

### License
Apache 2.0

## 🏗️ How to use - Contracts
##### 💫 Build
Use these [instructions](https://use.ink/getting-started/setup) to set up your ink!/Rust environment    
Run this command in the contract folder:

```sh
cargo contract build
```

##### 💫 Run unit test

```sh
cargo test
```
##### 💫 Deploy
First start your local node. You can do that by running `make up` in the root directory of the project.

##### 💫 Run integration test
Execute `make test` in the root directory of the project.