use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1", author = "devtbit", about = "Wallet CLI")]
pub struct Opts {
    #[clap(short = 't', long = "testnet", about = "Use testnet network")]
    pub with_testnet: bool,
    #[clap(short = 'r', long = "regtest", about = "Use regtest network")]
    pub with_regtest: bool,
    #[clap(long, about = "Export private and public encrypted master keys")]
    pub export: bool,
    #[clap(long = "label", about = "Label for files with private and public encrypted master keys")]
    pub label: Option<String>,
    #[clap(short = 's', long, value_name = "N", about = "Use Shamir Shares")]
    pub shamir_shares: Option<u8>,
    #[clap(long = "mnemonic", value_name = "WORDS", about = "Inline mnemonic")]
    pub with_mnemonic: Option<String>,
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    #[clap(about = "Manage addresses")]
    Address(AddressCommand),
    #[clap(about = "Manage master account")]
    Master(MasterCommand),
    #[clap(about = "Fetch data from blockchain")]
    Get(GetCommand),
}

#[derive(Clap)]
pub struct MasterCommand {
    #[clap(subcommand)]
    pub subcommand: MasterSubCommand
}

#[derive(Clap)]
pub enum MasterSubCommand {
    #[clap(about = "New master account")]
    New(NewMasterSubCommand),
    #[clap(about = "Recover master account from mnemonic")]
    Recover(RecoverMasterSubCommand),
    #[clap(about = "Show master xpubkey (hex)")]
    Pubkey(PubkeyMasterSubCommand),
}

#[derive(Clap)]
pub struct NewMasterSubCommand {
    #[clap(short, long, value_name = "E", about = "Entropy level for private keys (1-3)", required = false, default_value = "1")]
    pub entropy: i32,
    #[clap(long, value_name = "N", about = "Min value for N-of-M rules", required = false, default_value = "0")]
    pub min: u8,
    #[clap(long, value_name = "M", about = "Max value for N-of-M rules", required = false, default_value = "0")]
    pub max: u8,
    #[clap(short = 's', long, about = "Use Shamir Secret-Sharing SLIP-0039 (needs min & max for N-of-M rule)")]
    pub shamir_sharing: bool,
}

#[derive(Clap)]
pub struct RecoverMasterSubCommand {
    #[clap(short = 's', long, value_name = "N", about = "To recover from N number of Shamir Secret-Sharing (SLIP-0039) Shares")]
    pub shamir_shares: Option<u8>,
}

#[derive(Clap)]
pub struct PubkeyMasterSubCommand {}

#[derive(Clap)]
pub struct AddressCommand {
    #[clap(long = "type", value_name = "TYPE", about = "Type of address")]
    pub address_type: Option<String>,
    #[clap(short = 'n', long = "number", value_name = "ACCOUNT", about = "Account number", default_value = "0")]
    pub account_number: u32,
    #[clap(short, long = "sub", value_name = "SUBACCOUNT", about = "Sub account number", default_value = "0")]
    pub subaccount: u32,
    #[clap(short, long = "kix", value_name = "K", about = "Key derivation instance", default_value = "0")]
    pub kix: u32,
    #[clap(long = "script-hash", value_name = "HASH", about = "The script hash to use for P2WSH addresses")]
    pub script_hash: Option<String>,
    #[clap(subcommand)]
    pub subcommand: AddressSubCommand,
}

#[derive(Clap)]
pub enum AddressSubCommand {
    #[clap(about = "Generate address")]
    Generate(GenerateAddressSubCommand),
}

#[derive(Clap)]
pub struct GenerateAddressSubCommand {}

#[derive(Clap)]
pub struct GetCommand {
    #[clap(long = "rpc", value_name = "ENDPOINT", about = "Connection string to a full node through RPC (with the following format USER:PASSWORD@[http:https]://ADDRESS:PORT")]
    pub rpc: String,
    #[clap(subcommand)]
    pub subcommand: GetSubCommand,
}

#[derive(Clap)]
pub enum GetSubCommand {
    #[clap(about = "Get ")]
    Coins(CoinsGetSubCommand),
}

#[derive(Clap)]
pub struct CoinsGetSubCommand {
    #[clap(long, value_name = "ADDR", about = "Address to fetch the coins from")]
    pub address: Option<String>,
    #[clap(short = 'n', long = "number", value_name = "ACCOUNT", about = "Account number")]
    pub account_number: Option<u32>,
    #[clap(short, long = "sub", value_name = "SUBACCOUNT", about = "Sub account number")]
    pub subaccount: Option<u32>,
    #[clap(short, long = "kix", value_name = "K", about = "Key derivation instance")]
    pub kix: Option<u32>,
    #[clap(long = "rescan", about = "Rescan the blockchain")]
    pub rescan: bool,
    #[clap(long, about = "Block height starting point to start scanning for coins")]
    pub start_block: Option<usize>,
    #[clap(long, about = "Block height end point to start scanning for coins")]
    pub end_block: Option<usize>,
    #[clap(long, value_name = "BLOCKS", about = "Scan specified number of blocks from the last one")]
    pub last: Option<usize>,
    #[clap(long = "type", value_name = "TYPE", about = "Type of address")]
    pub address_type: Option<String>,
    #[clap(long, about = "Display amounts in Sats")]
    pub sats: bool,
    #[clap(long, value_name = "MAX", about = "Limit the number of coins to fetch")]
    pub limit: Option<i32>,
    #[clap(long, about = "Target amount to be gathered")]
    pub amount: Option<String>,
    #[clap(long, about = "Flag to look for coins in descending order")]
    pub desc: bool,
}

pub fn validate_opts(opts: &Opts, _: bool) -> Result<&str, &str> {
    if opts.with_testnet && opts.with_regtest {
        return Err("testnet or regtest, only specify one.");
    }

    if opts.export && opts.label == None {
        return Err("Need to specify a label for export file");
    }

    match &opts.subcommand {
        SubCommand::Address(cmd_opts) => {
            match &cmd_opts.subcommand {
                AddressSubCommand::Generate(_) => {
                    if let Some(addr) = &cmd_opts.address_type {
                        if addr.to_lowercase() == "p2wsh"/* && cmd_opts.script_hash == None */{
                            return Err("P2WSH not implemented yet");
                        }
                    }
                }
            }
        },
        SubCommand::Master(cmd_opts) => {
            match &cmd_opts.subcommand {
                MasterSubCommand::New(sub_opts) => {
                    if sub_opts.shamir_sharing {
                        if sub_opts.min == 0 || sub_opts.max == 0 {
                            return Err("Min and max need to be specified for Shamir Sharing");
                        } else if sub_opts.min > sub_opts.max {
                            return Err("Min should be less in value than max");
                        }
                    }
                },
                MasterSubCommand::Recover(sub_opts) => {
                    if let Some(s) = sub_opts.shamir_shares {
                        if s < 2 {
                            return Err("Invalid min value for Shamir Shares");
                        }
                    }
                },
                MasterSubCommand::Pubkey(_) => {},
            }
        },
        SubCommand::Get(sub_opts) => {
            match &sub_opts.subcommand {
                GetSubCommand::Coins(o) => {
                    if !o.rescan {
                        if let Some(_) = o.start_block {
                            return Err("Missing --rescan argument for start block option");
                        } else if let Some(_) = o.end_block {
                            return Err("Missing --rescan argument for end block option");
                        } else if let Some(_) = o.last {
                            return Err("Missing --rescan argument for last option");
                        }
                    } else {
                        if let Some(_) = o.last {
                            if let Some(_) = o.start_block {
                                return Err("Cannot specify last argument with start block");
                            } else if let Some(_) = o.end_block {
                                return Err("Cannot specify last argument with end block");
                            }
                        }
                    }
                    if let Some(_) = o.address {
                        if let Some(_) = o.account_number {
                            return Err("Cannot specify address and account number");
                        } else if let Some(_) = o.subaccount {
                            return Err("Cannot specify address and sub account number");
                        } else if let Some(_) = o.kix {
                            return Err("Cannot specify address and key derivation instance");
                        } else if let Some(_) = o.address_type {
                            return Err("Cannot specify address type, it will be inferred from the input address");
                        }
                    } else {
                        if o.account_number == None {
                            return Err("Need to specify at least an address or an account number");
                        }
                    }
                    if let Some(_) = o.limit {
                        if let Some(_) = o.amount {
                            return Err("Cannot specify both limit and a target amount");
                        }
                    }
                },
            }
        },
    }
    Ok("OK")
}