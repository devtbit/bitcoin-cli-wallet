use std::str::FromStr;

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
use bitcoincore_rpc::{
    bitcoin::{self as bcore},
};
use master::Master;

mod cli;
mod core;
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

    if opts.with_testnet {
        network = Network::Testnet;
    } else if opts.with_regtest {
        network = Network::Regtest;
    }

    let label = match &opts.label {
        None => "wallet",
        Some(l) => l
    };

    match &opts.subcommand {
        cli::SubCommand::Address(cmd_opts) => {
            let (master_acc, password) = init_master(&opts, network);
            let address_type: AccountAddressType = parse_address_type(cmd_opts.address_type.clone());

            let n = cmd_opts.account_number;
            let m = cmd_opts.subaccount;
            let k = cmd_opts.kix;
            match &cmd_opts.subcommand {
                cli::AddressSubCommand::Generate(_) => {
                    let pk = master_acc.get_child_pk(password.clone(), address_type, n, m, k).unwrap();
                    let addr = match address_type {
                        AccountAddressType::P2PKH => Address::p2pkh(&pk, network),
                        AccountAddressType::P2SHWPKH => Address::p2shwpkh(&pk, network),
                        AccountAddressType::P2WPKH | _ => Address::p2wpkh(&pk, network),
                    };
                    println!("{}{}Address:{}{} {} {}         ",
                        style::Bold, color::Fg(color::Blue), style::Reset, color::Fg(color::Blue),
                        addr, style::Reset
                    );
                    io::show_qr(addr.to_string());
                }
            }
        },
        cli::SubCommand::Master(cmd_opts) => {
            match &cmd_opts.subcommand {
                cli::MasterSubCommand::New(sub_opts) => {
                    let master_acc: Master;
                    let (password, success) = io::get_secret(
                        "Enter your password: ",
                        Some("Confirm your password: ")
                    );
                    if !success {
                        utils::fatal_kill("Failed to get password!");
                    }
                    let entropy = match sub_opts.entropy {
                        0 | 1   => MasterKeyEntropy::Sufficient,
                        2       => MasterKeyEntropy::Double,
                        3 | _   => MasterKeyEntropy::Paranoid,
                    };
                    if sub_opts.shamir_sharing {
                        master_acc = Master::new_with_ss(password.clone(), entropy, network, sub_opts.min, sub_opts.max).unwrap();
                    } else {
                        master_acc = Master::new(password.clone(), entropy, network).unwrap();
                    }
                    if opts.export {
                        master_acc.export_master(password.clone(), label);
                    }
                },
                cli::MasterSubCommand::Recover(_) => {
                    let master_acc: Master;
                    let (password, success) = io::get_secret(
                        "Enter your password: ",
                        Some("Confirm your password: ")
                    );
                    if !success {
                        utils::fatal_kill("Failed to get password!");
                    }
                    if let Some(words) = opts.with_mnemonic {
                        master_acc = Master::new_from_inline_mnemonic(words.clone(), password.clone(), network).unwrap();
                    } else {
                        master_acc = Master::new_from_mnemonic(password.clone(), network, opts.shamir_shares).unwrap();
                    }
                    if opts.export {
                        master_acc.export_master(password.clone(), label);
                    }
                },
                cli::MasterSubCommand::Pubkey(_) => {
                    let (master_acc, _) = init_master(&opts, network);
                    let mpk = master_acc.get_master_public();
                    println!("{}", mpk);
                    io::show_qr(format!("{}", mpk).to_string());
                }
            }
        },
        cli::SubCommand::Get(cmd_opts) => {
            match &cmd_opts.subcommand {
                cli::GetSubCommand::Coins(sub_opts) => {

                    let mut node = core::Node::from_connection_string(cmd_opts.rpc.clone()).unwrap();
                    node.connect(network.to_string()).unwrap();

                    // let mut coins: Vec<ListUnspentResultEntry>;
                    // let mut total: bcore::Amount;
                    let address: bcore::Address;

                    let _network = match network {
                        Network::Bitcoin => bcore::Network::Bitcoin,
                        Network::Testnet => bcore::Network::Testnet,
                        Network::Regtest => bcore::Network::Regtest,
                    };

                    if let Some(addr) = &sub_opts.address {
                        node.load_watchonly_wallet(label);
                        address = bcore::Address::from_str(addr).unwrap();
                        node.import(label, Some(address.clone()), None).unwrap();
                    } else {
                        let (master_acc, password) = init_master(&opts, network);
                        let n = match sub_opts.account_number {
                            Some(a) => a,
                            None => 0,
                        };
                        let sub = match sub_opts.subaccount {
                            Some(sa) => sa,
                            None => 0,
                        };
                        let kix = match sub_opts.kix {
                            Some(k) => k,
                            None => 0,
                        };

                        let address_type: AccountAddressType = parse_address_type(sub_opts.address_type.clone());
                        
                        let _pk = master_acc.get_child_pk(password.clone(), address_type, n, sub, kix).unwrap();
                        let pk = bcore::PublicKey::from_str(&_pk.to_string()).unwrap();
                        address = match address_type {
                            AccountAddressType::P2PKH => bcore::Address::p2pkh(&pk, _network),
                            AccountAddressType::P2SHWPKH => bcore::Address::p2shwpkh(&pk, _network).unwrap(),
                            AccountAddressType::P2WPKH | _ => bcore::Address::p2wpkh(&pk, _network).unwrap(),
                        };
                        node.load_watchonly_wallet(label);
                        node.import(label, None, Some(&pk)).unwrap();
                    }

                    if sub_opts.rescan {
                        if let Err(e) = node.rescan(sub_opts.last, sub_opts.start_block, sub_opts.end_block) {
                            node.unload(Some(label));
                            utils::fatal_kill(&e);
                        }
                    }

                    match node.get_coins(sub_opts.limit, sub_opts.amount.clone(), Some(&vec![&address]), sub_opts.desc) {
                        Ok((coins, total)) => io::show_coins(&coins, total, sub_opts.sats),
                        Err(_) => println!("Failed to fetch coins"),
                    };

                    node.unload(Some(label));
                }
            }
        },
    }
}

fn parse_address_type(addr: Option<String>) -> AccountAddressType {
    match &addr {
        Some(addr) => {
            match addr.as_str() {
                "p2wpkh" => AccountAddressType::P2WPKH,
                "p2wsh" => AccountAddressType::P2WSH(4711),
                "p2pkh" => AccountAddressType::P2PKH,
                "p2shwpkh" | _ => AccountAddressType::P2SHWPKH,
            }
        },
        None => AccountAddressType::P2SHWPKH,
    }
}

fn init_master(opts: &cli::Opts, network: Network) -> (Master, String) {
    let (password, success) = io::get_secret(
        "Enter your password: ",
        Some("Confirm your password: ")
    );
    if !success {
        utils::fatal_kill("Failed to get password!");
    }
    if let Some(label) = &opts.label {
        (Master::new_from_encrypted_files(label, password.clone()).unwrap(), password)
    } else if let Some(words) = &opts.with_mnemonic {
        (Master::new_from_inline_mnemonic(words.to_string(), password.clone(), network).unwrap(), password)
    } else {
        (Master::new_from_mnemonic(password.clone(), network, opts.shamir_shares).unwrap(), password)
    }
}