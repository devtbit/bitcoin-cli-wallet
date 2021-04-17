# Bitcoin CLI Wallet

###### WARNING: Do not use this with real coins unless you verified the code and know what you are doing!

Simple CLI with Bitcoin wallet functionality. Part of my learning process in programming with Bitcoin & the Rust language. This is a work in progress, it supports mainnet, testnet and regtest.

### Build

```
cargo build
```

### Example usage

#### Wallet creation

To create a new wallet and save it encrypted to disk:
```
wallet --label mywallet --export master new
```
Note the 2 generated files on current folder, these need to be present to reuse this wallet

By default it has entropy level of 1 so this shows you a 12 word seed, you can increase the entropy level to have 24 or 48 words (2 and 3 respectively):
```
wallet master new -e 2
```

You can also generate a split seed with Shamir Secret-Sharing SLIP-0039:
```
wallet master new -s --min 3 --max 5
```
This will generate 5 seed shares of which you can use 3 to recover the wallet

Show master xpubkey:
```
wallet --label mywallet master pubkey
```

#### Address generation

To generate an address from the new wallet:
```
wallet --label mywallet address generate
```
By default this generates an address with BIP32 path `m/49' /0'/0' /0/0`

You can generate custom paths:
```
wallet --label mywallet address -n 10 -s 500 -k 100 generate
```
This generates an address with BIP32 path `m/49' /0'/10' /500/100`

Generate an address without writing keys to disk (it will prompt for mnemonic):
```
wallet address generate
```

You can also specify mnemonics inline:
###### WARNING: Do not send coins to this address you will lose them!
```
wallet --mnemonic "shove stage useful observe gospel bachelor decorate tiny swallow exhibit remember pepper" address generate
```

#### Fetch coins

You can fetch your coins from a Bitcoin Core node with the following command (must be a full node running with -txindex=1):
###### WARNING: Always use your own nodes, you will expose your privacy by sharing public keys
```
wallet --label mywallet get --rpc "myuser:mypassword@https://mynode.address:8332" coins --rescan -n 0 --last 50000
```
This command will connect to the node, create a watch-only wallet with the public key of your account with BIP32 Path `m/49' /0'/0' /0/0`.
It will then rescan the las 50,000 blocks of the blockchain looking for unspent outputs.

You can also fetch coins from an address, without generating a wallet (testnet example, address grabbed randomly from blockchain):
```
 $ > wallet -t get --rpc "myuser:mypassword@https://mynode.address:18332" coins --rescan --address="mxVFsFW5N4mu1HPkxPttorvocvzeZ7KZyk" --start-block 100000
-------------------------------------------------------------------
b587d8b26a6a57c8bc144495fec0a074d94dd1287f6a4ffe778d542d6aa45e9f:3
-------------------------------------------------------------------
1:      16.27719944 BTC (1 confirmations)
-------------------------------------------------------------------
TOTAL: 16.27719944 BTC
```
Coins will be shown with the amount, confirmations and a header displaying the transaction id and the output index (txid:vout). You can use `--sats` option to display amounts in sats.
If you know more or less the block heights where your coins might be you can specify limits through `--last`, `--start-block` and `--end-block` options

If you want to fetch your coins again but you know the node already scanned the blockchain you can ommit the `--rescan` option.
The `--label` option will determine which name to use on the node for the watch-only wallet.

For more options and commands:
```
wallet help
```

### TODO list:

- Generate and sign transactions
- Scripts support (smart contracts)