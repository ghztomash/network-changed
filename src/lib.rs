use network_state::NetworkState;
use public_ip_address::lookup::LookupProvider;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod network_state;

const DEFAULT_EXPIRE_TIME: u64 = 3600;

#[derive(Debug, PartialEq)]
pub struct NetworkObserver {
    config: ObserverConfig,
    last_state: NetworkState,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ObserverConfig {
    pub expire_time: u64,
    pub persist: bool,
    pub all_interfaces: bool,
    pub public_address: bool,
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
        }
    }

    pub fn default() -> Self {
        Self {
            expire_time: DEFAULT_EXPIRE_TIME,
            persist: false,
            all_interfaces: false,
            public_address: false,
        }
    }

    pub fn enable_public_address(&mut self, public_address: bool) -> &Self {
        self.public_address = public_address;
        self
    }

    pub fn enable_all_interfaces(&mut self, all_interfaces: bool) -> &Self {
        self.all_interfaces = all_interfaces;
        self
    }

    pub fn enable_persist(&mut self, persist: bool) -> &Self {
        self.persist = persist;
        self
    }
}

#[derive(Debug, PartialEq)]
pub enum NetworkChange {
    None,
    Expired,
    DefaultInterface,
    SecondaryInterface,
    PublicAddress,
}

impl NetworkObserver {
    pub fn new(config: ObserverConfig) -> Self {
        let mut current_state = if config.persist {
            NetworkState::new()
        } else {
            NetworkState::new()
        };
        // expire the state
        current_state.last_update -= Duration::from_secs(DEFAULT_EXPIRE_TIME);

        NetworkObserver {
            config,
            last_state: current_state,
        }
    }

    pub fn current_state(&self) -> NetworkState {
        let mut current_state = NetworkState::new();
        if self.config.all_interfaces {
            current_state.all_interfaces = Some(netdev::get_interfaces());
        }
        if self.config.public_address {
            if let Ok(response) = public_ip_address::perform_cached_lookup_with(
                vec![
                    (LookupProvider::MyIpCom, None),
                    (LookupProvider::GetJsonIp, None),
                    (LookupProvider::Ipify, None),
                    (LookupProvider::IpInfo, None),
                ],
                None,
                Some(5),
                false,
            ) {
                current_state.public_address = Some(response.ip);
            }
        }
        current_state
    }

    pub fn state_change(&mut self) -> NetworkChange {
        let current_state = self.current_state();
        let state_changed = self.last_state.compare(&current_state, &self.config);

        if state_changed != NetworkChange::None {
            //update state
            self.last_state = current_state;
        }
        state_changed
    }

    pub fn state_did_change(&mut self) -> bool {
        self.state_change() != NetworkChange::None
    }
}

impl Drop for NetworkObserver {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let config = ObserverConfig::default();
        let mut observer = NetworkObserver::new(config);
        assert_eq!(observer.state_change(), NetworkChange::Expired);
    }
}
