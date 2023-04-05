use std::env::var;

pub fn img_dir() -> String {
    // This only runs in one environment, we can safely
    // assume that it won't panic
    let mut default_dir = home::home_dir().unwrap();
    default_dir.push("modkit_images");
    var("MODKIT_IMG_DIR").unwrap_or(format!("{}", default_dir.display()))
}

pub fn light_gpio_pins() -> [u8; 4] {
    [21, 22, 27, 17]
}

pub fn contact_sensor_pin() -> u8 {
    18
}