use chrono::{Local, Timelike};
use colored::*;
use network_changed::{NetworkObserver, ObserverConfig};

fn main() {
    env_logger::init();
    let config = ObserverConfig::default()
        .enable_observe_all_interfaces(true)
        .enable_observe_public_address(true)
        .enable_persist(true);
    let mut observer = NetworkObserver::new(config);

    let state = observer.state_change();
    let now = Local::now();

    println!(
        "{} - Network changed: {}",
        format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second(),).bold(),
        format!("{:?}", state).blue().bold()
    );
}
