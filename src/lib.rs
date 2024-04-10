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
    pub all_interfaces: bool,
    pub public_address: bool,
}

impl ObserverConfig {
    pub fn new(expire_time: u64, all_interfaces: bool, public_address: bool) -> Self {
        Self {
            expire_time,
            all_interfaces,
            public_address,
        }
    }

    pub fn default() -> Self {
        Self {
            expire_time: 3600,
            all_interfaces: false,
            public_address: false,
        }
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
    pub fn new(all_interfaces: bool, public_address: bool) -> Self {
        let mut current_state = NetworkState::new();
        // expire the state
        current_state.last_update -= Duration::from_secs(DEFAULT_EXPIRE_TIME);

        let config = ObserverConfig::new(DEFAULT_EXPIRE_TIME, all_interfaces, public_address);
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
        let mut observer = NetworkObserver::new(false, false);
        assert_eq!(observer.state_change(), NetworkChange::Expired);
    }
}
