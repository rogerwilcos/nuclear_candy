use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use num_cpus;
use rayon::prelude::*;
use secp256k1::{rand::rngs::JitterRng, PublicKey, Secp256k1};
use serde_json::{json, Value};
use std::fs::{read_to_string, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Error, Read, Write};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tiny_keccak::keccak256;
use web3::types::Address;

const ETHRICH_LEN: i32 = 21932810;

#[tokio::main]
async fn main() {
    println!("{}", "Bot started");

    // let _ = sort_addressed();

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

            let pb = ProgressBar::new(ETHRICH_LEN as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:100.cyan/blue} {pos}/{len} ({percent}%)")
                    .expect("Failed to create progress bar")
                    .progress_chars("##-"),
            );

            // Filtering part...
            let address_string = format!("{:?}", pub_address);

            if binary_search(&address_string) {
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
                let _ = file.read_to_string(&mut contents);

                // Converts existing data to a JSON array
                let mut data: Vec<Value> =
                    serde_json::from_str(&contents).unwrap_or_else(|_| vec![]);

                // Adds the new data to the JSON array
                data.push(json_data);

                // Writes the data back to the file in updated form
                let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open("addresses.json");
                writeln!(file.unwrap(), "{:?}", serde_json::to_string_pretty(&data)).unwrap();
            }
        }
    });

    Ok(())
}

fn _create_address(reader: &mut BufReader<File>) -> Result<()> {
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

    for line in reader.lines() {
        if line.unwrap().contains(&address_string) {
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

    Ok(())
}

fn binary_search(record: &String) -> bool {
    let now: Instant = Instant::now();
    let file_source = OpenOptions::new()
        .read(true)
        .open("ethrich_source_sorted.txt")
        .unwrap();

    let reader = BufReader::new(file_source);
    let lines = reader.lines();
    let address_vestor: Vec<String> = lines.collect::<Result<_, _>>().unwrap();

    if let Some(index) = get_address_index(&record, &address_vestor) {
        let elapsed = now.elapsed().as_micros();
        println!(
            "Binary search index {} of {} with TRUE took: {} microseconds",
            index, address_vestor[index], elapsed
        );
        true
    } else {
        // let elapsed = now.elapsed().as_micros();
        // println!("Binary search with FAILE took: {} microseconds", elapsed);
        false
    }
}

fn get_address_index(name: &String, array: &Vec<String>) -> Option<usize> {
    match array.binary_search(name) {
        Ok(index) => Some(index),
        Err(_) => None,
    }
}

fn _sort_addressed() -> Result<(), Error> {
    let now = Instant::now();

    let wordlist = read_to_string("ethrich_unsorted.txt")?;
    let mut list: Vec<&str> = wordlist.split_ascii_whitespace().collect();
    println!("ethrich list len: {}", list.len());

    list.sort_unstable();

    let mut writer = BufWriter::new(File::create("ethrich_sorted.txt")?);
    writer.write(list.join("\n").as_bytes())?;

    let elapsed = now.elapsed().as_micros();
    println!("Took {} microseconds", elapsed);

    Ok(())
}
