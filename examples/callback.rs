use chrono::{Local, Timelike};
use colored::*;
use network_changed::{
    network_state::NetworkState, NetworkChange, NetworkObserver, ObserverConfig,
};
use std::{thread, time};

fn on_change_callback(state: &NetworkChange, old: &NetworkState, new: &NetworkState) {
    let description = match state {
        NetworkChange::DefaultInterface => {
            let old = old
                .default_interface
                .as_ref()
                .map(|i| i.name.as_str())
                .unwrap_or("None");
            let new = new
                .default_interface
                .as_ref()
                .map(|i| i.name.as_str())
                .unwrap_or("None");
            format!("Default interface changed: {} -> {}", old, new)
        }
        NetworkChange::SecondaryInterface => "Secondary interface changed".to_string(),
        NetworkChange::PublicAddress => {
            let old = old
                .observe_public_address
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or("None".to_string());
            let new = new
                .observe_public_address
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or("None".to_string());
            format!("Public address changed: {} -> {}", old, new)
        }
        NetworkChange::Expired => "State expired".to_string(),
        _ => "".to_string(),
    };
    let now = Local::now();
    println!(
        "{} - Network status changed: {} - {}",
        format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second(),).bold(),
        format!("{:?}", state).blue().bold(),
        description
    );
}

fn main() {
    let sleep_time = time::Duration::from_millis(100);

    let config = ObserverConfig::default()
        .enable_observe_public_address(true)
        .set_on_change(on_change_callback);
    let mut observer = NetworkObserver::new(config);
    loop {
        _ = observer.state_change();
        thread::sleep(sleep_time);
    }
}
