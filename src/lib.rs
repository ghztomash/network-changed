use netdev::Interface;
use std::time::SystemTime;
use std::net::IpAddr;

pub struct NetworkObserver {
    all_interfaces: bool,
    public_address: bool,
    last_state: Option<NetworkState>,
}

pub struct NetworkState {
    last_update: SystemTime,
    default_interface: Interface,
    all_interfaces: Option<Vec<Interface>>,
    public_address: Option<IpAddr>,
}

impl NetworkState {
    pub fn new() -> Self {
        Self { last_update: SystemTime::now(), default_interface: netdev::get_default_interface().unwrap(), all_interfaces: None, public_address: None }
    }
}

impl NetworkObserver {
    pub fn new(all_interfaces: bool, public_address: bool) -> Self {
        let current_state = NetworkState::new();
        NetworkObserver {
            all_interfaces,
            public_address,
            last_state: None,
        }
    }
    
    pub fn state_changed(&mut self) -> bool {
        let current_state = NetworkState::new();
        //update state 
        self.last_state = Some(current_state);
        true
    }
}

impl Drop for NetworkObserver {
    fn drop(&mut self) {
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let observer = NetworkObserver::new(false, false);
        assert!(observer.state_changed());
    }
}
