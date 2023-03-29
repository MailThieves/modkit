use modkit::prelude::*;
use std::thread::sleep;
use std::time::Duration;
use log::*;


fn main() {
    env_logger::init();

    info!("Starting light test");

    light::set(true).expect("set light");
    sleep(Duration::from_millis(100));
    light::set(false).expect("set light");
    sleep(Duration::from_millis(100));
    light::set(true).expect("set light");
    sleep(Duration::from_millis(100));

    let mut state = true;
    loop {
        info!("Toggling light");
        info!("Light is on? {:?}", light::is_on());
        light::set(state).expect("set light");
        sleep(Duration::from_secs(2));
        state = !state;
    }
}