use data_encoding::HEXUPPER;
use ring::error::Unspecified;
use ring::rand::SecureRandom;
use ring::{digest, pbkdf2, rand};
use std::num::NonZeroU32;

use ring::pbkdf2::PBKDF2_HMAC_SHA512;

pub fn salt_hash_password_with_pbkdf2() -> Result<(), Unspecified> {
    const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
    let n_iter = NonZeroU32::new(100_000).unwrap();
    let rng = rand::SystemRandom::new();

    let mut salt = [0u8; CREDENTIAL_LEN];
    rng.fill(&mut salt)?;

    let password = "iamazy";
    let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        PBKDF2_HMAC_SHA512,
        n_iter,
        &salt,
        password.as_bytes(),
        &mut pbkdf2_hash,
    );

    println!("Salt:{}", HEXUPPER.encode(&salt));
    println!("PBKDF2 hash:{}", HEXUPPER.encode(&pbkdf2_hash));

    let should_succeed = pbkdf2::verify(
        PBKDF2_HMAC_SHA512,
        n_iter,
        &salt,
        password.as_bytes(),
        &pbkdf2_hash,
    );

    let wrong_password = "image";
    let should_failed = pbkdf2::verify(
        PBKDF2_HMAC_SHA512,
        n_iter,
        &salt,
        wrong_password.as_bytes(),
        &pbkdf2_hash,
    );

    println!("{:?}", should_succeed.is_ok());
    println!("{:?}", !should_failed.is_ok());

    Ok(())
}
