use network_changed::NetworkObserver;
use std::io;

fn main() {
    let mut input = String::new();
    let mut observer = NetworkObserver::new(false, false);
    loop {
        println!("Network status changed: {}", observer.state_changed());
        _ = io::stdin().read_line(&mut input);
    }
}
