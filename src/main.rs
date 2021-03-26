use clap::Clap;
use termion::{
    color,
    style
};
use bitcoin_wallet::{
    account::{
        AccountAddressType,
        MasterKeyEntropy,
    },
};
use bitcoin::{Address, network::constants::Network};

mod cli;
mod crypto;
mod io;
mod master;
mod utils;

fn main() {
    let opts: cli::Opts = cli::Opts::parse();

    if let Err(o) = cli::validate_opts(&opts, true) {
        utils::fatal_kill(o);
    }

    let mut network = Network::Bitcoin;

    let (password, success) = io::get_secret(
        "Enter your password",
        Some("Confirm your password")
    );

    if !success {
        utils::fatal_kill("Failed to get password!");
    }

    if opts.with_testnet {
        network = Network::Testnet;
    } else if opts.with_regtest {
        network = Network::Regtest;
    }

    let master_acc: master::Master;

    match &opts.subcommand {
        cli::SubCommand::Address(cmd_opts) => {
            if let Some(prefix) = opts.prefix {
                master_acc = master::Master::new_from_encrypted_files(&prefix, password.clone()).unwrap();
            } else if let Some(words) = opts.with_mnemonic {
                master_acc = master::Master::new_from_inline_mnemonic(words.clone(), password.clone(), network).unwrap();
            } else {
                master_acc = master::Master::new_from_mnemonic(password.clone(), network, opts.shamir_shares).unwrap();
            }

            let address_type: AccountAddressType = match &cmd_opts.address_type {
                Some(addr) => {
                    match addr.as_str() {
                        "p2wpkh" => AccountAddressType::P2WPKH,
                        "p2wsh" => AccountAddressType::P2WSH(4711),
                        "p2pkh" => AccountAddressType::P2PKH,
                        "p2shwpkh" | _ => AccountAddressType::P2SHWPKH,
                    }
                },
                None => AccountAddressType::P2SHWPKH,
            };

            let n = cmd_opts.account_number;
            let m = cmd_opts.subaccount;
            let k = cmd_opts.kix;
            match &cmd_opts.subcommand {
                cli::AddressSubCommand::Generate(_) => {
                    let mut acc = master_acc.new_account(password.clone(), address_type, n, m);
                    let pk = acc.compute_base_public_key(k).unwrap();
                    let addr = match address_type {
                        AccountAddressType::P2PKH => Address::p2pkh(&pk, network),
                        AccountAddressType::P2SHWPKH => Address::p2shwpkh(&pk, network),
                        _ | AccountAddressType::P2WPKH => Address::p2wpkh(&pk, network),
                    };
                    println!("{}{}Address:{}{} {} {}         ",
                        style::Bold, color::Fg(color::Blue), style::Reset, color::Fg(color::Blue),
                        addr, style::Reset
                    );
                }
            }
        },
        cli::SubCommand::Master(cmd_opts) => {
            match &cmd_opts.subcommand {
                cli::MasterSubCommand::New(sub_opts) => {
                    let entropy = match sub_opts.entropy {
                        0 | 1   => MasterKeyEntropy::Sufficient,
                        2       => MasterKeyEntropy::Double,
                        3 | _   => MasterKeyEntropy::Paranoid,
                    };
                    if sub_opts.shamir_sharing {
                        master_acc = master::Master::new_with_ss(password.clone(), entropy, network, sub_opts.min, sub_opts.max).unwrap();
                    } else {
                        master_acc = master::Master::new(password.clone(), entropy, network).unwrap();
                    }
                },
                cli::MasterSubCommand::Recover(_) => {
                    if let Some(words) = opts.with_mnemonic {
                        master_acc = master::Master::new_from_inline_mnemonic(words.clone(), password.clone(), network).unwrap();
                    } else {
                        master_acc = master::Master::new_from_mnemonic(password.clone(), network, opts.shamir_shares).unwrap();
                    }
                }
            }
            if opts.export {
                master_acc.export_master(password.clone(), &opts.prefix.unwrap());
            }
        }
    }
}