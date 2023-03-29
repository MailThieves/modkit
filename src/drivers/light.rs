#[cfg(not(feature = "hardware"))]
#[allow(unused)]
pub mod light {
    use super::super::DeviceError;
    use log::*;

    pub fn set(_state: bool) -> Result<(), DeviceError> {
        error!("You tried to use the light outside of hardware mode");
        error!("Recompile with `--features hardware`");
        Ok(())
    }

    pub fn is_on() -> Result<bool, DeviceError> {
        error!("You tried to use the light outside of hardware mode");
        error!("Recompile with `--features hardware`");
        Ok(false)
    }
}

#[cfg(feature = "hardware")]
pub mod light {
    use super::super::DeviceError;
    use rppal::gpio::Gpio;
    use log::*;

    const LIGHT_GPIO_PIN: u8 = 21;

    pub fn set(state: bool) -> Result<(), DeviceError> {
        trace!("Connecting to pin {:?}", LIGHT_GPIO_PIN);
        let mut light_pin = Gpio::new()?.get(LIGHT_GPIO_PIN)?.into_output();
        if state {
            trace!("Setting light pin {LIGHT_GPIO_PIN} high");
            light_pin.set_high();
        } else {
            trace!("Setting light pin {LIGHT_GPIO_PIN} low");
            light_pin.set_low();
        }
        Ok(())
    }

    pub fn toggle() -> Result<(), DeviceError> {
        trace!("Connecting to pin {:?}", LIGHT_GPIO_PIN);
        let mut light_pin = Gpio::new()?.get(LIGHT_GPIO_PIN)?.into_output();
        trace!("Toggling pin {:?}", LIGHT_GPIO_PIN);
        light_pin.toggle();
        Ok(())
    }

    pub fn is_on() -> Result<bool, DeviceError> {
        trace!("Connecting to pin {:?}", LIGHT_GPIO_PIN);
        let mut light_pin = Gpio::new()?.get(LIGHT_GPIO_PIN)?.into_output();
        let state = light_pin.is_set_high();
        trace!("Found pin to be high? {state}");
        Ok(state)
    }
}

#[cfg(test)]
#[cfg(feature = "hardware")]
mod tests {
    use super::*;

    #[test]
    fn test_on_off() {
        assert!(light::set(true).is_ok());
        let mut status = light::is_on().unwrap();
        assert!(status);
        assert!(light::set(false).is_ok());
        status = light::is_on().unwrap();
        assert!(!status);
    }
}
