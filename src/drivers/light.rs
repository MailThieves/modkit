pub mod light {
    use super::super::DeviceError;
    use crate::drivers::hardware_enabled;
    use rppal::gpio::Gpio;
    use log::*;

    const LIGHT_GPIO_PIN: u8 = 21;

    pub fn set(state: bool) -> Result<(), DeviceError> {
        error!("Called!");
        if !hardware_enabled() {
            warn!("You tried to use the light when hardware is not enabled");
            return Ok(());
        }

        trace!("Connecting to pin {:?}", LIGHT_GPIO_PIN);
        let mut light_pin = Gpio::new()?.get(LIGHT_GPIO_PIN)?.into_output();
        // trace!("GPIO reports Pin #{}", light_pin.pin());
        if state {
            trace!("Setting light pin {LIGHT_GPIO_PIN} high");
            light_pin.set_high();
        } else {
            trace!("Setting light pin {LIGHT_GPIO_PIN} low");
            light_pin.set_low();
        }

        Ok(())
    }

    pub fn is_on() -> Result<bool, DeviceError> {
        if !hardware_enabled() {
            warn!("You tried to use the light when hardware is not enabled");
            return Ok(false);
        }

        trace!("Connecting to pin {:?}", LIGHT_GPIO_PIN);
        let light_pin = Gpio::new()?.get(LIGHT_GPIO_PIN)?.into_output();
        let state = light_pin.is_set_high();
        trace!("Found pin to be high? {state}");
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use crate::drivers::hardware_enabled;

    use super::*;

    #[test]
    fn test_on_off() {
        if hardware_enabled() {
            assert!(light::set(true).is_ok());
            let mut status = light::is_on().unwrap();
            assert!(status);
            assert!(light::set(false).is_ok());
            status = light::is_on().unwrap();
            assert!(!status);
        }
    }
}
