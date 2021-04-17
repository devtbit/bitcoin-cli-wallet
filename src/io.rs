use std::{
    io::{
        stdin,
        stdout,
        Write,
    },
    process,
};
use termion::input::TermRead;
use bitcoin_wallet::mnemonic::Mnemonic;
use bitcoincore_rpc::{
    bitcoin::Amount,
    json::ListUnspentResultEntry,
};
use qrcode::{
    QrCode,
    render::unicode,  
};

pub fn show_new_mnemonic_from_words(mnemonic: &Vec<&str>) {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let stdin = stdin();
    let mut stdin = stdin.lock();
    stdout.write_all(b"Write down your menmonic words in a safe place. After you write down a word, hit enter to continue to the next word:\n").unwrap();
    stdout.flush().unwrap();

    let mnemonic_iter = mnemonic.iter();
    let mut i = 0;
    for word in mnemonic_iter {
        stdout.write_all(format!("{}: {}                        [PRESS ENTER TO CONTINUE]          ", i+1, word).as_bytes()).unwrap();
        stdout.flush().unwrap();
        let y = stdin.read_passwd(&mut stdout);
        if let Ok(Some(_)) = y {
            i = i + 1;
            stdout.write_all(b"\r").unwrap();
            stdout.flush().unwrap();
        } else {
            stdout.write_all(b"\nFailed to read input! Discard your words and try again\n").unwrap();
            process::exit(1);
        }
    }
    stdout.write_all(b"                                                                     \n").unwrap();
}

pub fn show_new_mnemonic(mnemonic: &Mnemonic) {
    let mnemonic_str = mnemonic.to_string();
    let words: Vec<&str> = mnemonic_str.split(" ").collect();
    show_new_mnemonic_from_words(&words);
}

pub fn get_secret(prompt: &str, confirm: Option<&str>) -> (String, bool) {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let stdin = stdin();
    let mut stdin = stdin.lock();

    stdout.write_all(prompt.as_bytes()).unwrap();
    stdout.flush().unwrap();
    let pass = stdin.read_passwd(&mut stdout);
    let password: String;
    if let Ok(Some(pass)) = pass {
        password = pass;
        stdout.write_all(b"\n").unwrap();
    } else {
        stdout.write_all(b"\nFailed to read input!\n").unwrap();
        return ("".to_string(), false);
    }

    if let Some(c) = confirm {
        stdout.write_all(c.as_bytes()).unwrap();
        stdout.flush().unwrap();
        let confirm_pass = stdin.read_passwd(&mut stdout);

        if let Ok(Some(confirm_pass)) = confirm_pass {
            if password.eq(&confirm_pass) {
                stdout.write_all(b"\n").unwrap();
                return (password, true);
            } else {
                stdout.write_all(b"\nInput did not match!\n").unwrap();
            }
        } else {
            stdout.write_all(b"\nFailed to read_input!\n").unwrap();
        }
    } else {
        return (password, true);
    }

    ("".to_string(), false)
}

pub fn show_coins(coins: &Vec<ListUnspentResultEntry>, total: Amount, as_sats: bool) {
    let den = match as_sats {
        true => "sats",
        false => "BTC",
    };
    fn show_line() {
        for _ in 0..67 {
            print!("-");
        }
    }
    let mut i = 0;
    for coin in coins {
        let amount = match as_sats {
            true => coin.amount.as_sat().to_string(),
            false => coin.amount.as_btc().to_string(),
        };
        show_line();
        println!("\n{}:{}", coin.txid, coin.vout);
        show_line();
        println!("\n{}:\t{} {} ({} confirmations)", i+1, amount, den, coin.confirmations);
        show_line();
        print!("\n");
        i = i + 1;
    }
    let aggregate = match as_sats {
        true => total.as_sat().to_string(),
        false => total.as_btc().to_string(),
    };
    println!("TOTAL: {} {}", aggregate, den);
}

pub fn show_qr(data: String) {
    let qr = QrCode::new(data).unwrap();
    let img = qr.render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    println!("{}", img);
}