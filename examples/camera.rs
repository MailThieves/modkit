#![allow(unused)]
use log::*;
use std::thread::sleep;
use std::time::Duration;

use rascam;

use modkit::prelude::*;

fn main() {
    env_logger::init();
    error!("Be sure to use info log level");
    info!("Starting camera example");

    let info = rascam::info().expect("camera info");

    if info.cameras.len() < 1 {
        error!("Found 0 cameras. Exiting");
        std::process::exit(1);
    }

    info!("{}", info);

    if let Some(cam_info) = info.cameras.get(0) {
        let mut camera = rascam::SimpleCamera::new(cam_info.clone()).expect("opening camera");
        camera.activate().expect("activate camera");

        sleep(Duration::from_secs(2));

        let bytes = camera.take_one().expect("taking a picture");
        File::create("image.jpg")
            .expect("creating file")
            .write_all(&bytes)
            .expect("write to file");

        info!("Saved image as image.jpg");
    }
}
