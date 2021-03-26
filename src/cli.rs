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
    #[clap(long = "master-prefix", about = "Prefix for files with private and public encrypted master keys")]
    pub prefix: Option<String>,
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
pub struct ShowMasterSubCommand {}

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

pub fn validate_opts(opts: &Opts, _: bool) -> Result<&str, &str> {
    if opts.with_testnet && opts.with_regtest {
        return Err("testnet or regtest, only specify one.");
    }

    if opts.export && opts.prefix == None {
        return Err("Need to specify a master prefix for export file");
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
            }
        },
    }
    Ok("OK")
}