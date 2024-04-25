use chrono::{Local, Timelike};
use colored::*;
use network_changed::{NetworkChange, NetworkObserver, ObserverConfig};
use std::{thread, time};

fn main() {
    env_logger::init();
    let sleep_time = time::Duration::from_millis(100);

    let config = ObserverConfig::default()
        .enable_observe_public_address(true)
        .enable_observe_all_interfaces(true);
    let mut observer = NetworkObserver::new(config);

    println!("Listenign for changes, press Ctrl+C to cancel...");
    loop {
        let state = observer.state_change();
        if state != NetworkChange::None {
            let now = Local::now();
            println!(
                "{} - Network changed: {}",
                format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second(),).bold(),
                format!("{:?}", state).blue().bold()
            );
        }
        thread::sleep(sleep_time);
    }
}
