use chrono::{Timelike, Utc};
use colored::*;
use network_changed::{NetworkObserver, ObserverConfig};

fn main() {
    let config = ObserverConfig::default()
        .enable_all_interfaces(true)
        .enable_public_address(true)
        .enable_persist(true);
    let mut observer = NetworkObserver::new(config);

    let state = observer.state_change();
    let now = Utc::now();

    println!(
        "{} - Network status changed: {}",
        format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second(),).bold(),
        format!("{:?}", state).blue().bold()
    );
}
