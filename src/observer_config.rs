use serde::{Deserialize, Serialize};

use crate::{network_state::NetworkState, NetworkChange};

pub const DEFAULT_EXPIRE_TIME: u64 = 3600;

type OnChangeCallback = fn(change: &NetworkChange, old: &NetworkState, new: &NetworkState);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ObserverConfig {
    pub expire_time: u64,
    pub persist: bool,
    pub all_interfaces: bool,
    pub public_address: bool,
    #[serde(skip)]
    pub on_change: Option<OnChangeCallback>,
}

impl ObserverConfig {
    pub fn new(
        expire_time: u64,
        persist: bool,
        all_interfaces: bool,
        public_address: bool,
    ) -> Self {
        Self {
            expire_time,
            persist,
            all_interfaces,
            public_address,
            on_change: None,
        }
    }

    pub fn default() -> Self {
        Self {
            expire_time: DEFAULT_EXPIRE_TIME,
            persist: false,
            all_interfaces: false,
            public_address: false,
            on_change: None,
        }
    }

    pub fn enable_public_address(mut self, public_address: bool) -> Self {
        self.public_address = public_address;
        self
    }

    pub fn enable_all_interfaces(mut self, all_interfaces: bool) -> Self {
        self.all_interfaces = all_interfaces;
        self
    }

    pub fn enable_persist(mut self, persist: bool) -> Self {
        self.persist = persist;
        self
    }

    pub fn set_expire_time(mut self, expire_time: u64) -> Self {
        self.expire_time = expire_time;
        self
    }

    pub fn set_on_change(mut self, callback: OnChangeCallback) -> Self {
        self.on_change = Some(callback);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config() {
        let config_new = ObserverConfig::new(360, true, true, true);
        let config_set = ObserverConfig::default()
            .set_expire_time(360)
            .enable_public_address(true)
            .enable_all_interfaces(true)
            .enable_persist(true);
        assert_eq!(config_new, config_set);
    }
}
