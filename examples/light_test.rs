use modkit::prelude::*;
use std::thread::sleep;
use std::time::Duration;
use log::*;


fn main() {
    env_logger::init();

    info!("Starting light test");

    // let mut state = true;
    light::set(true).expect("set light");
    sleep(Duration::from_millis(200));
    light::set(false).expect("Set light");
    sleep(Duration::from_millis(200));
    light::set(true).expect("set light");
    sleep(Duration::from_millis(200));
    light::set(false).expect("Set light");
    // loop {
    //     // info!("Light is on? {:?}", light::is_on());
        
    //     sleep(Duration::from_secs(2));
    //     state = !state;
    // }
}