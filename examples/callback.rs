use chrono::{Local, Timelike};
use colored::*;
use network_changed::{
    network_state::NetworkState, NetworkChange, NetworkObserver, ObserverConfig,
};
use std::{thread, time};

fn on_change_callback(state: &NetworkChange, old: &NetworkState, new: &NetworkState) {
    let description = match state {
        NetworkChange::DefaultInterface => {
            let old = old.default_interface.as_ref().unwrap().name.as_str();
            let new = new.default_interface.as_ref().unwrap().name.as_str();
            format!("Default interface changed from {} to {}", old, new)
        }
        NetworkChange::SecondaryInterface => "Secondary interface changed".to_string(),
        NetworkChange::PublicAddress => "Public address changed".to_string(),
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
        .enable_public_address(true)
        .set_on_change(on_change_callback);
    let mut observer = NetworkObserver::new(config);
    loop {
        _ = observer.state_change();
        thread::sleep(sleep_time);
    }
}
