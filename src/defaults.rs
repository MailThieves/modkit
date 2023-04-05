use std::env::var;

pub fn img_dir() -> String {
    var("MODKIT_IMG_DIR").unwrap_or(String::from("./img"))
}

pub fn light_gpio_pins() -> [u8; 4] {
    [21, 22, 27, 17]
}

pub fn contact_sensor_pin() -> u8 {
    18
}