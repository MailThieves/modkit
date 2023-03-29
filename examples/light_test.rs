use modkit::prelude::*;
use std::thread::sleep;
use std::time::Duration;
use log::*;


fn main() {
    env_logger::init();

    info!("Starting light test");

    let mut state = true;
    loop {
        light::set(state).expect("set light");
        info!("Light is on? {:?}", light::is_on());
        
        state = !state;
        sleep(Duration::from_secs(2));
    }
}