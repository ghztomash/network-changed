use log::{trace, warn};
pub use network_state::{Interfaces, NetworkState};
pub use observer_config::ObserverConfig;
use observer_config::DEFAULT_EXPIRE_TIME;
use public_ip_address::lookup::LookupProvider;
use std::time::Duration;

pub mod error;
pub mod network_interfaces;
pub mod network_state;
pub mod observer_config;

#[derive(Debug, PartialEq)]
pub struct NetworkObserver {
    config: ObserverConfig,
    last_state: NetworkState,
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
        let current_state = if config.persist {
            trace!("Loading state");
            NetworkState::load().unwrap_or_else(|_| NetworkState::new())
        } else {
            let mut s = NetworkState::new();
            // expire the state
            s.last_update -= Duration::from_secs(DEFAULT_EXPIRE_TIME);
            s
        };

        NetworkObserver {
            config,
            last_state: current_state,
        }
    }
}

#[cfg(not(feature = "async"))]
impl NetworkObserver {
    pub fn current_state(&self) -> NetworkState {
        let mut current_state = NetworkState::new();
        if self.config.observe_all_interfaces {
            current_state.all_interfaces = Some(Interfaces::new(netdev::get_interfaces()));
        }
        if self.config.observe_public_address {
            if let Ok(response) = public_ip_address::perform_cached_lookup_with(
                vec![
                    (LookupProvider::MyIpCom, None),
                    (LookupProvider::GetJsonIp, None),
                    (LookupProvider::Ipify, None),
                    (LookupProvider::IpInfo, None),
                ],
                None,
                Some(10),
                false,
            ) {
                current_state.public_address = Some(response.ip);
            } else {
                current_state.public_address = None;
                warn!("Failed to get public IP address");
            }
        }
        current_state
    }

    pub fn state_change(&mut self) -> NetworkChange {
        let current_state = self.current_state();
        let state_changed = self.last_state.compare(&current_state, &self.config);

        if state_changed != NetworkChange::None {
            // call on_change callback
            if let Some(callback) = self.config.on_change {
                callback(&state_changed, &self.last_state, &current_state);
            }
            //update state
            self.last_state = current_state;
        }
        state_changed
    }

    pub fn state_did_change(&mut self) -> bool {
        self.state_change() != NetworkChange::None
    }
}

#[cfg(feature = "async")]
impl NetworkObserver {
    pub async fn current_state(&self) -> NetworkState {
        let mut current_state = NetworkState::new();
        if self.config.observe_all_interfaces {
            current_state.all_interfaces = Some(netdev::get_interfaces());
        }
        if self.config.observe_public_address {
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
            } else {
                warn!("Failed to get public IP address");
            }
        }
        current_state
    }

    pub async fn state_change(&mut self) -> NetworkChange {
        let current_state = self.current_state().await;
        let state_changed = self.last_state.compare(&current_state, &self.config);

        if state_changed != NetworkChange::None {
            // call on_change callback
            if let Some(callback) = self.config.on_change {
                callback(&state_changed, &self.last_state, &current_state);
            }
            //update state
            self.last_state = current_state;
        }
        state_changed
    }

    pub async fn state_did_change(&mut self) -> bool {
        self.state_change().await != NetworkChange::None
    }
}

impl Drop for NetworkObserver {
    fn drop(&mut self) {
        if self.config.persist {
            trace!("Persisting state");
            _ = self.last_state.save();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "async"))]
    #[test]
    fn it_works() {
        let config = ObserverConfig::default();
        let mut observer = NetworkObserver::new(config);
        assert_eq!(observer.state_change(), NetworkChange::Expired);
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn it_works() {
        let config = ObserverConfig::default();
        let mut observer = NetworkObserver::new(config);
        assert_eq!(observer.state_change().await, NetworkChange::Expired);
    }
}
