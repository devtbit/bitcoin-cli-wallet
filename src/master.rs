use bitcoin_wallet::{
    mnemonic::Mnemonic,
    account::{
        Account,
        MasterAccount,
        Unlocker,
        AccountAddressType,
        MasterKeyEntropy,
    },
    sss::{
        ShamirSecretSharing,
        Share,
    },
    error::Error,
};
use bitcoin::{
    network::constants::Network,
    blockdata::{
        transaction::{SigHashType, Transaction, TxOut},
    },
};
use rand::Rng;
use termion::{
    color,
    style
};

use crate::io;
use crate::crypto;
use crate::utils;

pub struct Master {
    encrypted_master: MasterAccount,
    password: String,
}

impl Master {
    pub fn new(
        password: String,
        entropy: MasterKeyEntropy,
        network: Network,
    ) -> Result<Master, String> {
        let mnemonic = Mnemonic::new_random(entropy).unwrap();
        io::show_new_mnemonic(&mnemonic);
        let encrypted_master = MasterAccount::from_mnemonic(&mnemonic, 0, network, &password, None).unwrap();
        Ok(Master{
            encrypted_master,
            password,
        })
    }

    pub fn new_from_encrypted_files(prefix: &String, password: String) -> Result<Master, String> {
        let (encrypted, pubkey) = crypto::import_encrypted_keys(prefix, &password);
        let encrypted_master = MasterAccount::from_encrypted(&encrypted, pubkey, 0x0);
        Ok(Master {
            encrypted_master,
            password,
        })
    }

    pub fn new_from_mnemonic(
        password: String,
        network: Network,
        shamir_sharing: Option<u8>,
    ) -> Result<Master, String> {
        match shamir_sharing {
            Some(s) => Self::new_from_ss_mnemonic(password, network, s),
            None => {
                let (words, success) = io::get_secret("Enter your words: ", None);
                if success {
                    let mnemonic = Mnemonic::from_str(&words).unwrap();
                    let encrypted_master = MasterAccount::from_mnemonic(&mnemonic, 0, network, &password, None).unwrap();
                    return Ok(Master {
                        encrypted_master,
                        password,
                    });
                } else {
                    utils::fatal_kill("Failed to get words!");
                }
                return Err("Failed to init root from mnemonic".to_string());
            }
        }
    }

    pub fn new_from_inline_mnemonic(
        mnemonic: String,
        password: String,
        network: Network,
    ) -> Result<Master, String> {
        let mnemonic = Mnemonic::from_str(&mnemonic).unwrap();
        let encrypted_master = MasterAccount::from_mnemonic(&mnemonic, 0, network, &password, None).unwrap();
        Ok(Master {
            encrypted_master,
            password,
        })
    }

    fn new_from_ss_mnemonic(
        password: String,
        network: Network,
        n: u8,
    ) -> Result<Master, String> {
        let mut shares: Vec<Share> = vec![];
        for x in 0..n {
            println!("{}{}Input share #{}{}", style::Bold, color::Fg(color::Blue), x+1, style::Reset);
            let (words, success) = io::get_secret("Enter your words: ", None);
            if success {
                let share = Share::from_mnemonic(&words).unwrap();
                shares.push(share);
            } else {
                utils::fatal_kill("Failed to get words!");
            }
        }
        let seed = ShamirSecretSharing::combine(&shares, None).unwrap();
        let encrypted_master = MasterAccount::from_seed(&seed, 0, network, &password).unwrap();
        Ok(Master {
            encrypted_master,
            password,
        })
    }

    pub fn new_with_ss(
        password: String,
        entropy: MasterKeyEntropy,
        network: Network,
        n: u8,
        m: u8,
    ) -> Result<Master, String> {
        let encrypted_master = MasterAccount::new(entropy, network, &password).unwrap();
        let seed = encrypted_master.seed(network, &password).unwrap();
        let shares = ShamirSecretSharing::generate(1, &[(n, m)], &seed, None, 1).unwrap();
        let mut re_shares: Vec<Share> = Vec::new();
        let mut indexes: Vec<u8> = Vec::new();
        for _ in 0..n {
            let mut index = rand::thread_rng().gen_range(0..m);
            while indexes.iter().any(|&i| i == index) {
                index = rand::thread_rng().gen_range(0..m);
            }
            re_shares.push(shares[index as usize].clone());
            indexes.push(index);
        }
        let re_seed = ShamirSecretSharing::combine(&re_shares, None).unwrap();
        let re_master = MasterAccount::from_seed(&re_seed, 0, network, &password).unwrap();
        assert_eq!(encrypted_master.master_public(), re_master.master_public());
        assert_eq!(encrypted_master.encrypted(), re_master.encrypted());
        let mut i = 0;
        for share in shares {
            let mnemonic = share.to_mnemonic();
            let words: Vec<&str> = mnemonic.split(" ").collect();
            println!("{}{}Showing share #{}{}", style::Bold, color::Fg(color::Blue), i+1, style::Reset);
            io::show_new_mnemonic_from_words(&words);
            println!("{}[DONE]{}                                                          ", color::Fg(color::Green), style::Reset);
            i = i+1;
        }
        Ok(Master {
            encrypted_master,
            password,
        })
    }

    pub fn get_master(&self) -> &MasterAccount {
        &self.encrypted_master
    }

    pub fn new_account(
        &self,
        address_type: AccountAddressType,
        n: u32,
        m: u32,
        look_ahead: u32,
    ) -> Account {
        let mut unlocker = Unlocker::new_for_master(&self.encrypted_master, &self.password).unwrap();
        Account::new(&mut unlocker, address_type, n, m, look_ahead).unwrap()
    }

    pub fn export_master(self, prefix: &str) {
        crypto::export_encrypted_keys(&self.encrypted_master, prefix, &self.password);
    }

    pub fn export_account(
        self,
        address_type: AccountAddressType,
        n: u32,
        m: u32,
        look_ahead: u32,
        prefix: Option<&String>
    ) {
        let mut unlocker = Unlocker::new_for_master(&self.encrypted_master, &self.password).unwrap();
        let acc = Account::new(&mut unlocker, address_type, n, m, look_ahead).unwrap();
        let mut acc_prefix: String = n.to_string();
        acc_prefix = acc_prefix + &m.to_string();
        if let Some(p) = prefix {
            acc_prefix = String::from(p);
        }
        let pubkey = acc.master_public();
        let privkey = unlocker.sub_account_key(address_type, n, m);
        if let Ok(privkey) = privkey {
            crypto::encrypt_and_export(&privkey, &pubkey, &acc_prefix, &self.password);
        } else {
            utils::fatal_kill("Failed to unlock private key");
        }
    }

    pub fn sign_outputs(
        self,
        tx: &mut Transaction,
        spend: &Vec<TxOut>,
    ) {
    }

    pub fn sign_output(
        self,
        tx: &mut Transaction,
        txout: TxOut,
    ) -> Result<usize, Error> {
        let mut unlocker = Unlocker::new_for_master(&self.encrypted_master, &self.password).unwrap();
        self.encrypted_master.sign(tx, SigHashType::All, &(|_| Some(txout.clone())), &mut unlocker)
    }
}