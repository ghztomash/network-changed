use netdev::Interface;
use public_ip_address::lookup::LookupProvider;
use std::net::IpAddr;
use std::time::Duration;
use std::time::SystemTime;

static DEFAULT_EXPIRE_TIME: u64 = 3600;

pub struct NetworkObserver {
    config: ObserverConfig,
    last_state: NetworkState,
}

pub struct ObserverConfig {
    pub expire_time: u64,
    pub all_interfaces: bool,
    pub routing_table: bool,
    pub public_address: bool,
}

impl ObserverConfig {
    pub fn new(
        expire_time: u64,
        all_interfaces: bool,
        routing_table: bool,
        public_address: bool,
    ) -> Self {
        Self {
            expire_time,
            all_interfaces,
            routing_table,
            public_address,
        }
    }

    pub fn default() -> Self {
        Self {
            expire_time: 3600,
            all_interfaces: false,
            routing_table: false,
            public_address: false,
        }
    }
}

#[derive(Debug)]
pub struct NetworkState {
    last_update: SystemTime,
    default_interface: Option<Interface>,
    all_interfaces: Option<Vec<Interface>>,
    routing_table: Option<Vec<String>>,
    public_address: Option<IpAddr>,
}

impl PartialEq for NetworkState {
    fn eq(&self, other: &Self) -> bool {
        self.default_interface == other.default_interface
    }
}

#[derive(Debug, PartialEq)]
pub enum NetworkChange {
    None,
    Expired,
    DefaultInterface,
    SecondaryInterface,
    RoutingTable,
    PublicAddress,
}

impl NetworkState {
    pub fn new() -> Self {
        Self {
            last_update: SystemTime::now(),
            default_interface: netdev::get_default_interface().ok(),
            all_interfaces: None,
            routing_table: None,
            public_address: None,
        }
    }

    fn compare(&self, other: &Self, config: &ObserverConfig) -> NetworkChange {
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
        if config.routing_table {
            if self.routing_table != other.routing_table {
                return NetworkChange::RoutingTable;
            }
        }
        if config.public_address {
            if self.public_address != other.public_address {
                return NetworkChange::PublicAddress;
            }
        }

        NetworkChange::None
    }
}

impl NetworkObserver {
    pub fn new(all_interfaces: bool, routing_table: bool, public_address: bool) -> Self {
        let mut current_state = NetworkState::new();
        // expire the state
        current_state.last_update -= Duration::from_secs(DEFAULT_EXPIRE_TIME);

        let config = ObserverConfig::new(DEFAULT_EXPIRE_TIME, all_interfaces, routing_table, public_address);
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
        if self.config.routing_table {
            current_state.routing_table = None;
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
        let mut observer = NetworkObserver::new(false, false, false);
        assert_eq!(observer.state_change(), NetworkChange::Expired);
    }
}
