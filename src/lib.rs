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

#[derive(Debug)]
pub struct NetworkState {
    last_update: SystemTime,
    default_interface: Option<Interface>,
    all_interfaces: Option<Vec<Interface>>,
    public_address: Option<IpAddr>,
}

impl PartialEq for NetworkState {
    fn eq(&self, other: &Self) -> bool {
        self.default_interface == other.default_interface
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

    fn compare(&self, other: &Self, config: &ObserverConfig) -> bool {
        // check expire time
        if other
            .last_update
            .duration_since(self.last_update)
            .unwrap_or_default()
            .as_secs()
            >= config.expire_time
        {
            println!("State expired");
            return true;
        }

        // check default interface
        if self.default_interface != other.default_interface {
            println!("Default interface changed");
            return true;
        }
        if config.all_interfaces {
            if self.all_interfaces != other.all_interfaces {
                println!("Non defalt interfaces changed");
                return true;
            }
        }
        if config.public_address {
            if self.public_address != other.public_address {
                println!("Public address changed");
                return true;
            }
        }

        return false;
    }
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

    pub fn state_changed(&mut self) -> bool {
        let current_state = self.current_state();
        let state_changed = self.last_state.compare(&current_state, &self.config);

        if state_changed {
            //update state
            self.last_state = current_state;
        }
        state_changed
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
        assert!(observer.state_changed());
    }
}
