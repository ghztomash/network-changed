use network_changed::{NetworkObserver, ObserverConfig};

fn main() {
    let config = ObserverConfig::default().enable_all_interfaces(true).enable_persist(true);
    let mut observer = NetworkObserver::new(config);
    println!("Network status changed: {:?}", observer.state_change());
}
