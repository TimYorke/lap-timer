use rppal::{gpio::Gpio, pwm, i2c::I2c};
use rppal::pwm::{Channel, Polarity, Pwm};
use std::{error::Error, thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new()?;
//    let pin_scl = gpio.get(1)?.into
    let i2c = I2c::new()?;
}
