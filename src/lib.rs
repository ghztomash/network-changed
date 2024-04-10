use netdev::Interface;
use public_ip_address::lookup::LookupProvider;
use std::net::IpAddr;
use std::time::SystemTime;

pub struct NetworkObserver {
    all_interfaces: bool,
    public_address: bool,
    last_state: NetworkState,
}

#[derive(Debug)]
pub struct NetworkState {
    last_update: SystemTime,
    default_interface: Interface,
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
            default_interface: netdev::get_default_interface().unwrap(),
            all_interfaces: None,
            public_address: None,
        }
    }
}

impl NetworkObserver {
    pub fn new(all_interfaces: bool, public_address: bool) -> Self {
        let mut current_state = NetworkState::new();
        if all_interfaces {
            current_state.all_interfaces = Some(netdev::get_interfaces());
        }
        if public_address {
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
        NetworkObserver {
            all_interfaces,
            public_address,
            last_state: current_state,
        }
    }

    pub fn state_changed(&mut self) -> bool {
        let current_state = NetworkState::new();
        let mut state_changed = false;

        if self.last_state.default_interface != current_state.default_interface {
            state_changed = true;
        }
        if self.all_interfaces {

        }
        if self.public_address {

        }
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
