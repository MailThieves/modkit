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

pub fn flip_vertical() -> bool {
    match var("MODKIT_FLIP_VERTICAL") {
        Ok(s) => return s == "1",
        Err(_) => return false,
    }
}

pub fn pin() -> u16 {
    // If we can read the pin from the env var...
    if let Ok(overwrite) = var("MODKIT_PIN") {
        // and we can parse it to a u16...
        if let Ok(parsed) = overwrite.parse() {
            // then return
            return parsed;
        }
    }

    // otherwise, this is the default
    6245
}
