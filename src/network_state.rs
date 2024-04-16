use super::{NetworkChange, ObserverConfig};
use crate::error::Result;
use directories::ProjectDirs;
use log::trace;
use netdev::Interface;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
    net::IpAddr,
    time::SystemTime,
};
pub use crate::network_interfaces::Interfaces;

#[cfg(feature = "encryption")]
use crate::error::Error;
#[cfg(feature = "encryption")]
use cocoon::Cocoon;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NetworkState {
    pub last_update: SystemTime,
    pub default_interface: Option<Interface>,
    pub all_interfaces: Option<Interfaces>,
    pub public_address: Option<IpAddr>,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkState {
    pub fn new() -> Self {
        Self {
            last_update: SystemTime::now(),
            default_interface: netdev::get_default_interface().ok(),
            all_interfaces: None,
            public_address: None,
        }
    }

    pub fn compare(&self, other: &Self, config: &ObserverConfig) -> NetworkChange {
        // check expire time
        if other
            .last_update
            .duration_since(self.last_update)
            .unwrap_or_default()
            .as_secs()
            >= config.expire_time
        {
            return NetworkChange::Expired;
        }

        // check default interface
        if self.default_interface != other.default_interface {
            return NetworkChange::DefaultInterface;
        }
        if config.observe_all_interfaces && self.all_interfaces != other.all_interfaces {
            return NetworkChange::SecondaryInterface;
        }
        if config.observe_public_address && self.public_address != other.public_address {
            return NetworkChange::PublicAddress;
        }

        NetworkChange::None
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let serialized = serde_json::to_string(&self)?;
        Ok(serialized.into_bytes())
    }

    pub fn decode(data: Vec<u8>) -> Result<Self> {
        let serialized = String::from_utf8(data).unwrap_or_default();
        let deserialized: Self = serde_json::from_str(&serialized)?;
        Ok(deserialized)
    }

    pub fn save(&self) -> Result<()> {
        let data = self.encode()?;

        #[cfg(feature = "encryption")]
        let data = encrypt(data)?;

        let mut file = File::create(get_data_path())?;
        file.write_all(&data)?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        trace!("Loading state from {}", get_data_path());
        let mut file = File::open(get_data_path())?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        #[cfg(feature = "encryption")]
        let data = decrypt(data)?;

        Self::decode(data)
    }
}

#[cfg(feature = "encryption")]
fn decrypt(data: Vec<u8>) -> Result<Vec<u8>> {
    trace!("Decrypting data");
    let password = mid::get(env!("CARGO_PKG_NAME")).unwrap_or("network_state".to_string());
    let cocoon = if cfg!(debug_assertions) {
        Cocoon::new(password.as_bytes()).with_weak_kdf()
    } else {
        Cocoon::new(password.as_bytes())
    };
    match cocoon.unwrap(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(Error::EncryptionError(format!("Error decrypting: {:?}", e))),
    }
}

#[cfg(feature = "encryption")]
fn encrypt(data: Vec<u8>) -> Result<Vec<u8>> {
    trace!("Encrypting data");
    let password = mid::get(env!("CARGO_PKG_NAME")).unwrap_or("network_state".to_string());
    let mut cocoon = if cfg!(debug_assertions) {
        Cocoon::new(password.as_bytes()).with_weak_kdf()
    } else {
        Cocoon::new(password.as_bytes())
    };
    match cocoon.wrap(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(Error::EncryptionError(format!("Error encrypting: {:?}", e))),
    }
}

pub fn get_data_path() -> String {
    let file_name = "state.cache";
    if let Some(base_dirs) = ProjectDirs::from("", "", env!("CARGO_PKG_NAME")) {
        let mut dir = base_dirs.data_dir();
        // Create directory if it doesn't exist
        if !dir.exists() && fs::create_dir_all(dir).is_err() {
            // If we can't create the data directory, fallback to cache directory
            dir = base_dirs.cache_dir();
            if !dir.exists() && fs::create_dir_all(dir).is_err() {
                dir = base_dirs.config_dir();
            }
        }
        if let Some(path) = dir.join(file_name).to_str() {
            return path.to_string();
        }
    };
    // If we can't get any directory, fallback to current directory
    file_name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let state = NetworkState::new();
        let encoded = state.encode().unwrap();
        let decoded = NetworkState::decode(encoded).unwrap();
        assert_eq!(state, decoded);
    }

    #[test]
    fn test_save_load() {
        dbg!(get_data_path());
        let state = NetworkState::new();
        state.save().unwrap();
        let loaded = NetworkState::load().unwrap();
        assert_eq!(state, loaded);
    }

    #[test]
    #[cfg(feature = "encryption")]
    fn test_encrypt_decrypt() {
        let data = b"hello world".to_vec();
        let encrypted = encrypt(data.clone()).unwrap();
        let decrypted = decrypt(encrypted).unwrap();
        assert_eq!(data, decrypted);
    }
}
