# Bitcoin CLI Wallet

###### WARNING: Do not use this with real coins unless you know what you are doing

Simple CLI with Bitcoin wallet functionality. Part of my learning process in programming with Bitcoin & the Rust language. This is a work in progress, right now it only supports account & address generation.

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
```
wallet --mnemonic "shove stage useful observe gospel bachelor decorate tiny swallow exhibit remember pepper" address generate
```

For more options and commands:
```
wallet help
```

### TODO list:

- Get balances / handle coins
- Generate and sign transactions
- Scripts support (smart contracts)