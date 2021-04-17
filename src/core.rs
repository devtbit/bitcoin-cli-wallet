use bitcoincore_rpc::{
    Auth,
    Client,
    Error,
    RpcApi,
    bitcoin::{
        Address,
        Amount,
        PublicKey
    },
    json::ListUnspentResultEntry
};

pub struct Node {
    url: String,
    auth: Auth,
    pub client: Option<Client>,
    blocks: usize,
}

impl Node {
    pub fn new(
        url: String,
        user: String,
        password: String,
    ) -> Node {
        let auth = Auth::UserPass(user, password);
        Node {
            url,
            auth,
            client: None,
            blocks: 0,
        }
    }

    // user:pass@[http/https]://address:port
    pub fn from_connection_string(
        conn: String,
    ) -> Result<Node, String> {
        let r = conn.split("@").collect::<Vec<&str>>();
        let creds = r[0].split(":").collect::<Vec<&str>>();
        Ok(Self::new(r[1].to_string(), creds[0].to_string(), creds[1].to_string()))
    }

    pub fn connect(&mut self, _: String) -> Result<(), Error> {
        let client = Client::new(self.url.clone(), self.auth.clone())?;
        let info = client.get_blockchain_info()?;
        /*if info.chain != chain {
            // ERR
        }*/
        self.client = Some(client);
        self.blocks = info.blocks as usize;
        Ok(())
    }

    pub fn load_watchonly_wallet(&self, label: &str) -> Result<(), String> {
        if let Some(client) = &self.client {
            if let Err(_) = client.create_wallet(label, Some(true), None, None, None) {
                if let Err(_) = client.load_wallet(label) {
                    println!("[+] Watch-only wallet already loaded");
                } else {
                    println!("[+] Loaded watch-only wallet");
                }
            } else {
                println!("[+] Created new watch-only wallet");
            }
            return Ok(());
        } else {
            return Err("Not connected to Node".to_string());
        }
    }

    pub fn rescan(&self, last: Option<usize>, start: Option<usize>, end: Option<usize>) -> Result<(), String> {
        if let Some(client) = &self.client {
            if last == None && start == None && end == None {
                println!("WARNING: You are rescanning the whole blockchain, this will take some hours to complete. This command will timeout, you can keep trying to fetch coins without rescan but it might not be accurate untill the rescan finishes.");
            }
            let res;
            if let Some(l) = last {
                if l > self.blocks {
                    return Err("Last argument exceeds the total number of blocks".to_string());
                }
                res = client.rescan_blockchain(Some(self.blocks - l), None);
            } else {
                if let Some(e) = end {
                    if e > self.blocks {
                        return Err("Less blocks on the blockchain than specified end height".to_string());
                    } else if let Some(s) = start {
                        if e - s > 400000 {
                            println!("WARNING: You are rescannig a lot of blocks, the node might take some minutes / hours to finish rescan. This command will timeout, you can keep trying to fetch coins without rescan but it might not be accurate untill the rescan finishes.")
                        }
                    }
                }
                res = client.rescan_blockchain(start, end);
            }
            match res {
                Ok(r) => {
                    if let Some(end) = r.1 {
                        println!("[+] Scanned {} blocks", end - r.0);
                    }
                },
                Err(e) => {
                    let errmsg = match e.to_string().contains("currently") {
                        true => "Node is running a rescan that will take some time to finish.".to_string(),
                        false => e.to_string(),
                    };
                    return Err(errmsg);
                },
            }
            return Ok(());
        } else {
            return Err("Not connected to Node".to_string());
        }
    }

    pub fn import(&self, label: &str, address: Option<Address>, pk: Option<&PublicKey>) -> Result<(), String> {
        if let Some(client) = &self.client {
            if let Some(addr) = address {
                client.import_address(&addr, Some(label), Some(false)).unwrap();
                return Ok(());
            } else if let Some(p) = pk {
                client.import_public_key(p, Some(label), Some(false)).unwrap();
                return Ok(());
            } else {
                return Err("Need to specify at least one of address or pk".to_string());
            }
        } else {
            return Err("Not connected to Node".to_string());
        }
    }

    pub fn get_coins(&self, limit: Option<i32>, target: Option<String>, addresses: Option<&[&Address]>, desc: bool) -> Result<(Vec<ListUnspentResultEntry>, Amount), String> {
        if let Some(client) = &self.client {
            let mut unspents = client.list_unspent(None, None, addresses, Some(true), None).unwrap();
            let mut coins: Vec<ListUnspentResultEntry> = vec![];
            if unspents.len() == 0 {
                return Err("No coins found".to_string());
            }
            unspents.sort_by(|a, b| {
                match desc {
                    true => b.amount.cmp(&a.amount),
                    false => a.amount.cmp(&b.amount),
                }
            });
            let mut i = 0;
            let mut total: f64 = 0.0;
            let target_amount: Option<Amount> = match target {
                Some(t) => Some(Amount::from_str_with_denomination(&t).unwrap()),
                None => None
            };
            for output in &unspents {
                total = total + output.amount.as_btc();
                coins.push(output.clone());
                if let Some(l) = limit {
                    if i + 1 == l {
                        break;
                    }
                } else if let Some(t) = target_amount {
                    if total >= t.as_btc() {
                        break;
                    }
                }
                i = i + 1;
            }
            if let Some(t) = target_amount {
                if total < t.as_btc() {
                    return Err("Not enough coins for target amount!".to_string());
                } else {
                    for x in (0..coins.len()).rev() {
                        let coin = &coins[x] as &ListUnspentResultEntry;
                        let amount = coin.amount.as_btc();
                        if total - amount >= t.as_btc() {
                            coins.remove(x);
                            total = total - amount;
                        }
                    }
                }
            }
            let aggregate_amount = Amount::from_btc(f64::trunc(total * 100000000.0) / 100000000.0).unwrap();
            Ok((coins, aggregate_amount))
        } else {
            Err("Not connected to Node".to_string())
        }
    }

    pub fn unload(&self, label: Option<&str>) -> Result<(), String> {
        if let Some(client) = &self.client {
            client.unload_wallet(label);
            Ok(())
        } else {
            Err("Not connected to Node".to_string())
        }
    }
}