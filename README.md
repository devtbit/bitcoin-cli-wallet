# Bitcoin CLI Wallet

###### WARNING: Do not use this with real coins unless you know what you are doing

Simple CLI with Bitcoin wallet functionality. Part of my learning process in programming with Bitcoin & the Rust language. This is a work in progress, right now it only supports account & address generation.

### Build

```
cargo build
```

### Example usage

To create a new wallet and save it encrypted to disk:
```
wallet --master-prefix mywallet --export master new
```

To generate an address from the new wallet:
```
wallet --master-prefix mywallet address generate
```

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

- Generate and sign transactions
- Scripts support