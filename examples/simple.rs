use network_changed::NetworkObserver;
use network_changed::ObserverConfig;
use std::io;

fn main() {
    let mut input = String::new();
    let config = ObserverConfig::default();
    let mut observer = NetworkObserver::new(config);
    loop {
        println!("Network status changed: {:?}", observer.state_change());
        match io::stdin().read_line(&mut input) {
            Ok(r) => {
                if r == 0 {
                    break;
                }
                match input.trim() {
                    "quit" | "q" => break,
                    _ => (),
                }
                input.clear();
            }
            Err(error) => {
                println!("error: {}", error);
                break;
            }
        }
    }
}
