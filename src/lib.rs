use log::{debug, trace, warn};
pub use network_state::{Interfaces, NetworkState};
pub use observer_config::ObserverConfig;
use observer_config::DEFAULT_EXPIRE_TIME;
use public_ip_address::lookup::LookupProvider;
use std::time::Duration;

pub mod error;
pub mod network_interfaces;
pub mod network_state;
pub mod observer_config;
pub mod routes;

#[derive(Debug, PartialEq)]
pub struct NetworkObserver {
    config: ObserverConfig,
    last_state: NetworkState,
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum NetworkChange {
    None,
    Expired,
    DefaultInterface,
    SecondaryInterface,
    DefaultRoute,
    RoutingTable,
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

impl NetworkObserver {
    #[maybe_async::maybe_async]
    pub async fn current_state(&self) -> NetworkState {
        let mut current_state = NetworkState::new();
        // update current state
        if self.config.observe_all_interfaces {
            current_state.all_interfaces = Some(Interfaces::new(netdev::get_interfaces()));
        }
        // get default route
        if self.config.observe_default_route {
            if let Ok(handle) = net_route::Handle::new() {
                if let Some(route) = handle.default_route().await.unwrap() {
                    debug!("Default route:\n{:?}", route);
                    current_state.default_route = Some(route.into())
                } else {
                    warn!("No default route");
                }
            } else {
                warn!("Failed to get route handle");
            }
        }
        // get all routes
        if self.config.observe_all_routes {
            if let Ok(handle) = net_route::Handle::new() {
                if let Ok(routes) = handle.list().await {
                    debug!("All routes:\n{:?}", routes);
                    current_state.all_routes = Some(routes.into_iter().map(|r| r.into()).collect());
                } else {
                    warn!("Failed to get all routes");
                }
            } else {
                warn!("Failed to get route handle");
            }
        }
        // get public address
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
            )
            .await
            {
                current_state.public_address = Some(response.ip);
            } else {
                current_state.public_address = None;
                warn!("Failed to get public IP address");
            }
        }
        current_state
    }

    #[maybe_async::maybe_async]
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

    #[maybe_async::maybe_async]
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

    #[maybe_async::test(feature = "blocking", async(not(feature = "blocking"), tokio::test))]
    async fn it_works() {
        let config = ObserverConfig::default();
        let mut observer = NetworkObserver::new(config);
        assert_eq!(observer.state_change().await, NetworkChange::Expired);
    }
}
