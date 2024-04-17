use chrono::{Local, Timelike};
use colored::*;
use network_changed::{NetworkObserver, ObserverConfig};
use std::io::{stdout, Write};
use std::{thread, time};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let sleep_time = time::Duration::from_millis(100);

    let config = ObserverConfig::default()
        .enable_observe_public_address(false)
        .set_on_change(|state, _, _| {
            let now = Local::now();
            println!(
                "\n{} - Network changed: {}",
                format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second(),).bold(),
                format!("{:?}", state).blue().bold()
            );
        });
    let mut observer = NetworkObserver::new(config);

    // observe in a separate thread
    tokio::spawn(async move {
        loop {
            _ = observer.state_change().await;
            thread::sleep(sleep_time);
        }
    });

    print!("Doing something very important...");
    loop {
        print!(".");
        stdout().flush().unwrap();
        thread::sleep(time::Duration::from_millis(1000));
    }
    Ok(())
}
