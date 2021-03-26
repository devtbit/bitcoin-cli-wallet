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
    encrypted: MasterAccount,
}

impl Master {
    pub fn new(
        password: String,
        entropy: MasterKeyEntropy,
        network: Network,
    ) -> Result<Master, String> {
        let mnemonic = Mnemonic::new_random(entropy).unwrap();
        io::show_new_mnemonic(&mnemonic);
        let encrypted = MasterAccount::from_mnemonic(&mnemonic, 0, network, &password, None).unwrap();
        Ok(Master{
            encrypted,
        })
    }

    pub fn new_from_encrypted_files(prefix: &String, password: String) -> Result<Master, String> {
        let (encrypted_master, pubkey) = crypto::import_encrypted_keys(prefix, &password);
        let encrypted = MasterAccount::from_encrypted(&encrypted_master, pubkey, 0x0);
        Ok(Master {
            encrypted,
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
                    if let Ok(master) = Self::new_from_inline_mnemonic(words, password, network) {
                        return Ok(master);
                    }
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
        let encrypted = MasterAccount::from_mnemonic(&mnemonic, 0, network, &password, None).unwrap();
        Ok(Master {
            encrypted,
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
        let encrypted = MasterAccount::from_seed(&seed, 0, network, &password).unwrap();
        Ok(Master {
            encrypted,
        })
    }

    pub fn new_with_ss(
        password: String,
        entropy: MasterKeyEntropy,
        network: Network,
        n: u8,
        m: u8,
    ) -> Result<Master, String> {
        let encrypted = MasterAccount::new(entropy, network, &password).unwrap();
        let seed = encrypted.seed(network, &password).unwrap();
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
        assert_eq!(encrypted.master_public(), re_master.master_public());
        assert_eq!(encrypted.encrypted(), re_master.encrypted());
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
            encrypted,
        })
    }

    pub fn get_master(&self) -> &MasterAccount {
        &self.encrypted
    }

    pub fn new_account(
        &self,
        password: String,
        address_type: AccountAddressType,
        n: u32,
        m: u32,
    ) -> Account {
        let mut unlocker = Unlocker::new_for_master(&self.encrypted, &password).unwrap();
        Account::new(&mut unlocker, address_type, n, m, 10).unwrap()
    }

    pub fn export_master(self, password: String, prefix: &str) {
        crypto::export_encrypted_keys(&self.encrypted, prefix, &password);
    }

    pub fn export_account(
        self,
        password: String,
        address_type: AccountAddressType,
        n: u32,
        m: u32,
        prefix: Option<&String>
    ) {
        let mut unlocker = Unlocker::new_for_master(&self.encrypted, &password).unwrap();
        let acc = Account::new(&mut unlocker, address_type, n, m, 10).unwrap();
        let mut acc_prefix: String = n.to_string();
        acc_prefix = acc_prefix + &m.to_string();
        if let Some(p) = prefix {
            acc_prefix = String::from(p);
        }
        let pubkey = acc.master_public();
        let privkey = unlocker.sub_account_key(address_type, n, m);
        if let Ok(privkey) = privkey {
            crypto::encrypt_and_export(&privkey, &pubkey, &acc_prefix, &password);
        } else {
            utils::fatal_kill("Failed to unlock private key");
        }
    }

    pub fn sign_output(
        self,
        password: String,
        tx: &mut Transaction,
        txout: TxOut,
    ) -> Result<usize, Error> {
        let mut unlocker = Unlocker::new_for_master(&self.encrypted, &password).unwrap();
        self.encrypted.sign(tx, SigHashType::All, &(|_| Some(txout.clone())), &mut unlocker)
    }
}