use anyhow::Result;
use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};
use std::fs::File;
use std::io::{BufReader, Read, Write};

use ring::error::Unspecified;
use ring::rand::SecureRandom;
use ring::{hmac, rand};

fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

pub fn calculate_sha256_digest_of_file() -> Result<()> {
    let path = "file.txt";
    let mut output = File::create(&path)?;
    write!(output, "We will generate a digest of this text")?;

    let input = File::open(&path)?;
    let reader = BufReader::new(input);
    let digest = sha256_digest(reader)?;

    println!("SHA-256 digest: {}", HEXUPPER.encode(digest.as_ref()));
    Ok(())
}

pub fn sign_verify_message_with_hmac_digest() -> Result<(), Unspecified> {
    let mut key_value = [0u8; 48];
    let rng = rand::SystemRandom::new();
    rng.fill(&mut key_value)?;
    let key = hmac::Key::new(hmac::HMAC_SHA256, &key_value);

    let message = "Legitimate and important message.";
    let signature = hmac::sign(&key, message.as_bytes());
    hmac::verify(&key, message.as_bytes(), signature.as_ref())?;

    Ok(())
}
