pub mod light {
    use super::super::DeviceError;
    use crate::drivers::hardware_enabled;
    use crate::defaults;
    use rppal::gpio::{Gpio, OutputPin};
    use log::*;

    pub fn set(state: bool) -> Result<(), DeviceError> {
        if !hardware_enabled() {
            warn!("You tried to use the light when hardware is not enabled");
            return Ok(());
        }

        let pin_numbers: [u8; 4] = defaults::light_gpio_pins();
        // There are always 4
        let mut pins: Vec<OutputPin> = vec![
            Gpio::new()?.get(pin_numbers[0])?.into_output(),
            Gpio::new()?.get(pin_numbers[1])?.into_output(),
            Gpio::new()?.get(pin_numbers[2])?.into_output(),
            Gpio::new()?.get(pin_numbers[3])?.into_output(),
        ];

        // This is important
        for pin in pins.iter_mut() {
            pin.set_reset_on_drop(false);
        }

        if state {
            trace!("Setting light pins {:?} high", pin_numbers);
            for pin in pins.iter_mut() {
                pin.set_high();
            }
        } else {
            trace!("Setting light pins {:?} low", pin_numbers);
            for pin in pins.iter_mut() {
                pin.set_low();
            }
        }

        Ok(())
    }

    pub fn is_on() -> Result<bool, DeviceError> {
        if !hardware_enabled() {
            warn!("You tried to use the light when hardware is not enabled");
            return Ok(false);
        }
        
        let pin_numbers = defaults::light_gpio_pins();
        
        trace!("Connecting to pins {:?}", pin_numbers);
        // We treat all pins as one, they should always be set the same
        let light_pin = Gpio::new()?.get(pin_numbers[0])?.into_output();
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
