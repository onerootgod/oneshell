use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use zeroize::Zeroize;

const MASTER_KEY_FILENAME: &str = "master.key";

pub fn load_or_create_master_key(app_data_dir: &Path) -> Result<String> {
    fs::create_dir_all(app_data_dir)?;
    let key_path = master_key_path(app_data_dir);

    if key_path.exists() {
        let key = fs::read_to_string(&key_path)?;
        return Ok(key.trim().to_owned());
    }

    let mut raw_key = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut raw_key);
    let encoded = STANDARD.encode(raw_key);
    raw_key.zeroize();

    let mut file = fs::File::create(&key_path)?;
    file.write_all(encoded.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(encoded)
}

pub fn encrypt_secret(master_key: &str, plaintext: &str) -> Result<String> {
    let cipher_key = derive_key_material(master_key, "server-password");
    let cipher = Aes256Gcm::new_from_slice(&cipher_key)?;
    let mut nonce_bytes = [0_u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .context("failed to encrypt secret")?;

    let mut payload = nonce_bytes.to_vec();
    payload.extend(ciphertext);
    Ok(STANDARD.encode(payload))
}

pub fn decrypt_secret(master_key: &str, payload: &str) -> Result<String> {
    let cipher_key = derive_key_material(master_key, "server-password");
    let cipher = Aes256Gcm::new_from_slice(&cipher_key)?;
    let decoded = STANDARD.decode(payload)?;

    if decoded.len() < 13 {
        return Err(anyhow!("encrypted secret payload is malformed"));
    }

    let (nonce_bytes, ciphertext) = decoded.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .context("failed to decrypt secret")?;

    String::from_utf8(plaintext).context("decrypted secret was not valid UTF-8")
}

pub fn derive_database_key(master_key: &str) -> String {
    STANDARD.encode(derive_key_material(master_key, "sqlcipher"))
}

fn derive_key_material(master_key: &str, context: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(master_key.as_bytes());
    hasher.update(b"::");
    hasher.update(context.as_bytes());
    let output = hasher.finalize();

    let mut key = [0_u8; 32];
    key.copy_from_slice(&output);
    key
}

fn master_key_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(MASTER_KEY_FILENAME)
}
