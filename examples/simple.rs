use chrono::{Timelike, Utc};
use colored::*;
use network_changed::{NetworkChange, NetworkObserver, ObserverConfig};
use std::{thread, time};

fn main() {
    let sleep_time = time::Duration::from_millis(100);

    let config = ObserverConfig::default().enable_all_interfaces(true);
    let mut observer = NetworkObserver::new(config);
    loop {
        let state = observer.state_change();
        if state != NetworkChange::None {
            let now = Utc::now();
            println!(
                "{} - Network status changed: {}",
                format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second(),).bold(),
                format!("{:?}", state).blue().bold()
            );
        }
        thread::sleep(sleep_time);
    }
}
