use super::{NetworkChange, ObserverConfig};
use base64::prelude::*;
use cocoon::{Cocoon, Error};
use directories::BaseDirs;
use netdev::Interface;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NetworkState {
    pub last_update: SystemTime,
    pub default_interface: Option<Interface>,
    pub all_interfaces: Option<Vec<Interface>>,
    pub public_address: Option<IpAddr>,
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
        if config.all_interfaces {
            if self.all_interfaces != other.all_interfaces {
                return NetworkChange::SecondaryInterface;
            }
        }
        if config.public_address {
            if self.public_address != other.public_address {
                return NetworkChange::PublicAddress;
            }
        }

        NetworkChange::None
    }

    pub fn encode(&self) -> Vec<u8> {
        let serialized = serde_json::to_string(&self).unwrap();
        println!("serialized = {:#?}", serialized);
        let encoded = BASE64_STANDARD.encode(serialized);
        println!("encoded = {:#?}", encoded);
        encoded.into_bytes()
    }

    pub fn decode(data: Vec<u8>) -> Self {
        let encoded = String::from_utf8(data).unwrap();
        println!("encoded = {:#?}", encoded);
        let serialized = String::from_utf8(BASE64_STANDARD.decode(encoded).unwrap_or_default())
            .unwrap_or_default();
        let deserialized: Self = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:#?}", deserialized);
        deserialized
    }

    pub fn save(&self) {
        let data = self.encode();
        let data = encrypt(data).unwrap();
        let mut file = File::create(get_data_path()).unwrap();
        file.write_all(&data).unwrap();
    }

    pub fn load() -> Self {
        let mut file = File::open(get_data_path()).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        let data = decrypt(data).unwrap();
        Self::decode(data)
    }
}

pub fn decrypt(data: Vec<u8>) -> Result<Vec<u8>, Error> {
    let password = b"pass";
    let cocoon = if cfg!(debug_assertions) {
        Cocoon::new(password).with_weak_kdf()
    } else {
        Cocoon::new(password)
    };
    cocoon.unwrap(&data)
}

pub fn encrypt(data: Vec<u8>) -> Result<Vec<u8>, Error> {
    let password = b"pass";
    let mut cocoon = if cfg!(debug_assertions) {
        Cocoon::new(password).with_weak_kdf()
    } else {
        Cocoon::new(password)
    };
    cocoon.wrap(&data)
}

pub fn get_data_path() -> String {
    if let Some(base_dirs) = BaseDirs::new() {
        let mut dir = base_dirs.data_dir();
        // Create directory if it doesn't exist
        if !dir.exists() && fs::create_dir_all(dir).is_err() {
            // If we can't create the data directory, fallback to cache directory
            dir = base_dirs.cache_dir();
            if !dir.exists() && fs::create_dir_all(dir).is_err() {
                dir = base_dirs.home_dir();
            }
        }
        if let Some(path) = dir.join(".networkstate").to_str() {
            return path.to_string();
        }
    };
    // If we can't get any directory, fallback to current directory
    ".networkstate".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let state = NetworkState::new();
        let encoded = state.encode();
        let decoded = NetworkState::decode(encoded);
        assert_eq!(state, decoded);
    }

    #[test]
    fn test_save_load() {
        dbg!(get_data_path());
        let state = NetworkState::new();
        state.save();
        let loaded = NetworkState::load();
        assert_eq!(state, loaded);
    }
}
