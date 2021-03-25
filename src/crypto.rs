use bitcoin_wallet::{
    account::MasterAccount,
    error::Error,
};
use crypto::{
    aes, blockmodes, buffer,
    buffer::{BufferResult, ReadBuffer, WriteBuffer},
    digest::Digest,
    sha2::Sha256,
};
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};

use crate::utils;

pub fn import_encrypted_keys(prefix: &str, password: &str) -> (Vec<u8>, ExtendedPubKey) {
    let pubfile = format!("{}-pub", prefix);
    let pub_bytes = utils::read_from_file(&pubfile);
    let decrypted_pub_bytes = decrypt(&pub_bytes, password).unwrap();
    let pub_str = String::from_utf8(decrypted_pub_bytes).unwrap();
    let obj = serde_json::from_str(&pub_str).unwrap();
    let master_bytes = utils::read_from_file(&prefix);
    (master_bytes, obj)
}

pub fn export_encrypted_keys(master: &MasterAccount, prefix: &str, password: &str) {
    utils::write_to_file(&prefix, master.encrypted());
    let pubfile = format!("{}-pub", prefix);
    let pubk = master.master_public();
    let serialized = serde_json::to_string(&pubk).unwrap();
    let serial_bytes = serialized.into_bytes();
    if let Ok(encrypted_bytes) = encrypt(serial_bytes.as_slice(), password) {
        utils::write_to_file(&pubfile, &encrypted_bytes);
    } else {
        utils::fatal_kill("Failed to encrypt and save keys");
    }
}

pub fn encrypt_and_export(privkey: &ExtendedPrivKey, pubkey: &ExtendedPubKey, prefix: &str, password: &str) {
    let pubfile = format!("{}-pub", prefix);
    let p_serial = serde_json::to_string(privkey).unwrap();
    let p_serial_bytes = p_serial.into_bytes();
    if let Ok(encrypted_bytes) = encrypt(p_serial_bytes.as_slice(), password) {
        utils::write_to_file(prefix, &encrypted_bytes);
    } else {
        utils::fatal_kill("Failed to encrypt and save keys");
    }
    let pub_serial = serde_json::to_string(pubkey).unwrap();
    let pub_serial_bytes = pub_serial.into_bytes();
    if let Ok(encrypted_bytes) = encrypt(pub_serial_bytes.as_slice(), password) {
        utils::write_to_file(&pubfile, &encrypted_bytes);
    } else {
        utils::fatal_kill("Failed to encrypt and save keys");
    }
}

/*
 *  As taken from rust-wallet::account::Seed
 */

 pub fn encrypt(data: &[u8], passphrase: &str) -> Result<Vec<u8>, Error> {
     let mut key = [0u8; 32];
     let mut sha2 = Sha256::new();
     sha2.input(passphrase.as_bytes());
     sha2.result(&mut key);

     let mut encryptor = aes::ecb_encryptor(aes::KeySize::KeySize256, &key, blockmodes::PkcsPadding {});
     let mut encrypted = Vec::new();
     let mut reader = buffer::RefReadBuffer::new(data);
     let mut buffer = [0u8; 1024];
     let mut writer = buffer::RefWriteBuffer::new(&mut buffer);
     loop {
         let result = encryptor.encrypt(&mut reader, &mut writer, true)?;
         encrypted.extend(
             writer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .map(|i| *i),
         );
         match result {
             BufferResult::BufferUnderflow => break,
             BufferResult::BufferOverflow => {}
         }
     }
     Ok(encrypted)
 }

 pub fn decrypt(encrypted: &[u8], passphrase: &str) -> Result<Vec<u8>, Error> {
     let mut key = [0u8; 32];
     let mut sha2 = Sha256::new();
     sha2.input(passphrase.as_bytes());
     sha2.result(&mut key);

     let mut decrypted = Vec::new();
     let mut reader = buffer::RefReadBuffer::new(encrypted);
     let mut buffer = [0u8; 1024];
     let mut writer = buffer::RefWriteBuffer::new(&mut buffer);
     let mut decryptor = aes::ecb_decryptor(aes::KeySize::KeySize256, &key, blockmodes::PkcsPadding {});
     loop {
         let result = decryptor.decrypt(&mut reader, &mut writer, true)?;
         decrypted.extend(
             writer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .map(|i| *i),
         );
         match result {
             BufferResult::BufferUnderflow => break,
             BufferResult::BufferOverflow => {}
         }
     }
     Ok(decrypted)
 }