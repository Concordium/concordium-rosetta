use anyhow::{Context, Error, Result};
use serde_json;
use std::env;
use std::io;

mod lib;

fn main() -> Result<()> {
    // Read keys from JSON file.
    let args: Vec<String> = env::args().collect();
    let path = match args.get(1) {
        None => return Err(Error::msg("key file not provided")),
        Some(p) => p,
    };
    let keys = lib::load_keys(path).context("cannot load account keys")?;

    // Read hash from stdin.
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("cannot read from stdin")?;
    let trimmed_input = input.trim();

    // // Debug output.
    // println!("Keys: {}", serde_json::to_string(&keys)?);
    // println!("Stdin: {}", input);

    // Sign hash using loaded keys.
    let tx_sig = lib::sign_hash(&keys, trimmed_input)?;

    // Print as JSON.
    let str = serde_json::to_string(&tx_sig).context("cannot serialize signatures into JSON")?;
    println!("{}", str);

    // // Print as text.
    // for (cred_idx, e) in tx_sig.signatures {
    //     println!("- credential index: {}", cred_idx.index);
    //     for (key_idx, key_sig) in e {
    //         let sig_str = hex::encode(&key_sig.sig);
    //         println!(
    //             " - key index: {}, signature: {}",
    //             key_idx.to_string(),
    //             sig_str
    //         );
    //     }
    // }

    Ok(())
}
