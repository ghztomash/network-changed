use chrono::{Local, Timelike};
use colored::*;
use network_changed::{NetworkObserver, ObserverConfig};
use std::error::Error;

#[cfg_attr(not(feature = "blocking"), tokio::main)]
#[maybe_async::maybe_async]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let config = ObserverConfig::default()
        .enable_observe_all_interfaces(true)
        .enable_observe_public_address(true)
        .enable_persist(true);
    let mut observer = NetworkObserver::new(config);

    let state = observer.state_change().await;
    let now = Local::now();

    println!(
        "{} - Network changed: {}",
        format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second(),).bold(),
        format!("{:?}", state).blue().bold()
    );
    Ok(())
}
