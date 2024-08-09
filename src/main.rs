use anyhow::Result;
use bip39::Mnemonic;
use indicatif::{ProgressBar, ProgressStyle};
use num_cpus;
use rayon::prelude::*;
use secp256k1::{rand::rngs::JitterRng, PublicKey, Secp256k1};
use serde_json::{json, Value};
use std::borrow::Borrow;
use std::fs::{read, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_keccak::keccak256;
use web3::types::Address;

#[tokio::main]
async fn main() {
    println!("{}", "Bot started");

    loop {
        let _ = run_bot().await;
        continue;
    }
}

async fn run_bot() -> Result<()> {

    (0..num_cpus::get()).into_par_iter().for_each(|_| {
        for _ in 0..1000 / num_cpus::get() {

            let secp = Secp256k1::new();
            let get_nstime = || -> u64 {
                let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                dur.as_secs() << 30 | dur.subsec_nanos() as u64
            };
            let mut rng = JitterRng::new_with_timer(get_nstime);
            let (secret_key, pub_key) = secp.generate_keypair(&mut rng);
            let public_key_address = |public_key: &PublicKey| -> Address {
                let public_key = public_key.serialize_uncompressed();
                debug_assert_eq!(public_key[0], 0x04);
                let hash = keccak256(&public_key[1..]);
                Address::from_slice(&hash[12..])
            };
            let pub_address = public_key_address(&pub_key);

            // Filtering part...
            let address_string = format!("{:?}", pub_address);

            let file = File::options().read(true).open("ethereum.txt").unwrap();
            let reader = BufReader::new(file);
            let lines = reader.lines();

            // let num_lines = reader.lines().count();
            // let pb = ProgressBar::new(num_lines as u64);
            // pb.set_style(
            //     ProgressStyle::default_bar()
            //         .template("[{elapsed_precise}] {bar:100.cyan/blue} {pos}/{len} ({percent}%)")
            //         .expect("Failed to create progress bar")
            //         .progress_chars("##-"),
            // );

            for line in lines {
                if (line.unwrap().contains(&address_string)) {
                    println!("found: {}", &address_string.to_owned());
                    let json_data = json!({
                        "secret_key": format!("{:?}", secret_key),
                        "public_key": format!("{:?}", pub_key),
                        "address": address_string
                    });

                    // Reads existing data from file
                    let mut file = File::open("addresses.json")
                        .unwrap_or_else(|_| File::create("addresses.json").unwrap());
                    let mut contents = String::new();
                    file.read_to_string(&mut contents);

                    // Converts existing data to a JSON array
                    let mut data: Vec<Value> =
                        serde_json::from_str(&contents).unwrap_or_else(|_| vec![]);

                    // Adds the new data to the JSON array
                    data.push(json_data);

                    // Writes the data back to the file in updated form
                    let mut file = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open("addresses.json");
                    writeln!(file.unwrap(), "{:?}", serde_json::to_string_pretty(&data));
                }
            }

            if address_string.starts_with("0x123456") {
                let json_data = json!({
                    "secret_key": format!("{:?}", secret_key),
                    "public_key": format!("{:?}", pub_key),
                    "address": address_string
                });

                // Reads existing data from file
                let mut file = File::open("addresses.json")
                    .unwrap_or_else(|_| File::create("addresses.json").unwrap());
                let mut contents = String::new();
                file.read_to_string(&mut contents);

                // Converts existing data to a JSON array
                let mut data: Vec<Value> =
                    serde_json::from_str(&contents).unwrap_or_else(|_| vec![]);

                // Adds the new data to the JSON array
                data.push(json_data);

                // Writes the data back to the file in updated form
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open("addresses.json");
                writeln!(file.unwrap(), "{:?}", serde_json::to_string_pretty(&data));
            }
        }
    });

    Ok(())
}

fn create_address(reader: &mut BufReader<File>) -> Result<()> {
    // Key pair generation and address calculation code...
    let secp = Secp256k1::new();
    let get_nstime = || -> u64 {
        let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        dur.as_secs() << 30 | dur.subsec_nanos() as u64
    };
    let mut rng = JitterRng::new_with_timer(get_nstime);
    let (secret_key, pub_key) = secp.generate_keypair(&mut rng);
    let public_key_address = |public_key: &PublicKey| -> Address {
        let public_key = public_key.serialize_uncompressed();
        debug_assert_eq!(public_key[0], 0x04);
        let hash = keccak256(&public_key[1..]);
        Address::from_slice(&hash[12..])
    };
    let pub_address = public_key_address(&pub_key);

    // Filtering part...
    let address_string = format!("{:?}", pub_address);

    // let mut rd = &mut reader;//.clone().to_owned() ;

    for line in reader.lines() {
        if (line.unwrap().contains(&address_string)) {
            let json_data = json!({
                "secret_key": format!("{:?}", secret_key),
                "public_key": format!("{:?}", pub_key),
                "address": address_string
            });

            // Reads existing data from file
            let mut file = File::open("addresses.json")
                .unwrap_or_else(|_| File::create("addresses.json").unwrap());
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            // Converts existing data to a JSON array
            let mut data: Vec<Value> = serde_json::from_str(&contents).unwrap_or_else(|_| vec![]);

            // Adds the new data to the JSON array
            data.push(json_data);

            // Writes the data back to the file in updated form
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open("addresses.json")?;
            writeln!(file, "{}", serde_json::to_string_pretty(&data)?)?;
        }
    }

    if address_string.starts_with("0x123456") {
        let json_data = json!({
            "secret_key": format!("{:?}", secret_key),
            "public_key": format!("{:?}", pub_key),
            "address": address_string
        });

        // Reads existing data from file
        let mut file = File::open("addresses.json")
            .unwrap_or_else(|_| File::create("addresses.json").unwrap());
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Converts existing data to a JSON array
        let mut data: Vec<Value> = serde_json::from_str(&contents).unwrap_or_else(|_| vec![]);

        // Adds the new data to the JSON array
        data.push(json_data);

        // Writes the data back to the file in updated form
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("addresses.json")?;
        writeln!(file, "{}", serde_json::to_string_pretty(&data)?)?;
    }

    Ok(())
}
