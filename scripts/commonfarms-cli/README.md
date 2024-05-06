## Command line tool for management of Common Farms

```
usage: commonfarms-cli [-h] [--chain URL] [--phrase SEED] [--farm-metadata PATH] [--psp22-metadata PATH] COMMAND ...

Command line tool for management of Common Farms

positional arguments:
  COMMAND
    create              create a new farm for given trading pool
    details             get details of an existing farm
    start               schedule a start of an existing farm
    stop                stop an existing farm
    withdraw            withdraw to the admin account all available balance of given token
    add-reward-token    add a new reward token to an existing farm
    increase-allowance  increase allowance for the farm to spend TOKEN

options:
  -h, --help            show this help message and exit
  --chain URL           WebSocker URL of the chain (possible shortcuts: mainnet, testnet, local)
  --phrase SEED         seed phrase of the farm admin account (if not supplied, an interactive prompt will ask for it)
  --farm-metadata PATH  path to farm contract metadata file
  --psp22-metadata PATH
                        path to PSP22 contract metadata file
```

For description of arguments required for each command please use a dedicated help command (e.g `commonfarms-cli start -h`).

**SECURITY WARNING** When interacting with production chains avoid using `--phrase` as it can leave the seed phrase in your cmd history. Skip the `--phrase` parameter and enter your phrase when prompted.