use chrono::{Local, Timelike};
use colored::*;
use network_changed::{
    network_state::Interfaces, network_state::NetworkState, NetworkChange, NetworkObserver,
    ObserverConfig,
};
use std::error::Error;
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
            format!("{} -> {}", old.yellow().bold(), new.yellow().bold())
        }
        NetworkChange::SecondaryInterface => {
            let old = old.all_interfaces.as_ref().unwrap();
            let new = new.all_interfaces.as_ref().unwrap();
            let diff = Interfaces::diff(old, new);
            let added = diff
                .added
                .to_owned()
                .keys()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(", ")
                .bold()
                .green();
            let updated = diff
                .updated
                .to_owned()
                .keys()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(", ")
                .bold()
                .yellow();
            let removed = diff
                .removed
                .to_owned()
                .keys()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(", ")
                .bold()
                .red();
            format!("~[{}], +[{}], -[{}]", updated, added, removed)
        }
        NetworkChange::PublicAddress => {
            let old = old
                .public_address
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or("None".to_string());
            let new = new
                .public_address
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or("None".to_string());
            format!("{} -> {}", old.yellow().bold(), new.yellow().bold())
        }
        NetworkChange::Expired => {
            let diff = new
                .last_update
                .duration_since(old.last_update)
                .unwrap_or_default();
            format!("{} seconds", diff.as_secs().to_string().yellow().bold())
        }
        _ => "".to_string(),
    };
    let now = Local::now();
    println!(
        "{} - Network change: {} - {}",
        format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second(),).bold(),
        format!("{:?}", state).blue().bold(),
        description
    );
}

#[cfg_attr(not(feature = "blocking"), tokio::main)]
#[maybe_async::maybe_async]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let sleep_time = time::Duration::from_millis(100);

    let config = ObserverConfig::default()
        .enable_observe_public_address(true)
        .enable_observe_all_interfaces(true)
        .set_on_change(on_change_callback);
    let mut observer = NetworkObserver::new(config);
    println!("Listenign for changes, press Ctrl+C to cancel...");
    loop {
        _ = observer.state_change();
        thread::sleep(sleep_time);
    }
}
