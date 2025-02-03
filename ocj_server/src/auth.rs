use std::{collections::HashMap, net::IpAddr};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

use rand::TryRngCore;

use crate::{error::{self, OcjError}, Result, config};

use config::{auth::Token, msg::Secure};


type Hasher = Sha256;

pub fn hash(s: &str) -> u128 {
    let mut hasher = Hasher::new();
    hasher.update(s);
    
    let mut hash_key = [0u8; 16];
    hash_key.copy_from_slice( &hasher.finalize()[0..16]);

    u128::from_le_bytes(hash_key)
}

pub fn gen_token() -> Result<Token> {
    let mut rng = rand::rngs::OsRng;
    let a= rng.try_next_u64().or_else(|e| Err(error::OcjError::RngCore(e)))? as u128;
    let b = rng.try_next_u64().or_else(|e| Err(error::OcjError::RngCore(e)))? as u128;
    Ok(Token(a * (std::u64::MAX as u128 + 1) + b))
}

pub struct Service {
    hash_key: u128, 
    tokens: Mutex<HashMap<IpAddr, Token>>, 
}

impl Service {
    pub fn init(key: &str) -> Self {
        Self {
            hash_key: hash(key),
            tokens: Mutex::new(HashMap::new()),
        }
    }
    pub async fn login(&self, ip: IpAddr, key: &str) -> Result<Token> {
        let hash_key = hash(key);
        if hash_key != self.hash_key {
            log::warn!("attempt login failed: ip: {ip}, key: {key}");
            return Err(error::OcjError::Auth(error::AuthError::IncorrectKey));
        }
        let token = gen_token()?;
        self.tokens.lock().await.insert(ip, token);
        log::info!("new token created for ip: {ip}");
        Ok(token)
    }

    pub async fn check_token(&self, ip: &IpAddr, token: &Token) -> Result<()> {
        let tokens = self.tokens.lock().await;
        let true_token = tokens.get(ip).ok_or(OcjError::Auth(error::AuthError::IpNotFound))?;
        if true_token != token {
            log::warn!("token check failed by ip: {ip}");
            Err(OcjError::Auth(error::AuthError::IncorrectToken))
        } else {
            Ok(())
        }
    }

    pub async fn unwrap_secure<T> (&self, ip: &IpAddr, msg: Secure<T>) -> Result<T> {
        let Secure {token, msg} = msg;
        self.check_token(ip, &token).await?;
        Ok(msg)
    }
} 