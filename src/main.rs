use rppal::{gpio::Gpio, pwm};
use rppal::pwm::{Channel, Polarity, Pwm};
use std::{error::Error, thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    let mut pin = Gpio::new()?.get(20)?.into_output();
    loop {
        pin.set_pwm(
            Duration::from_micros(100),
            Duration::from_micros(25)
        )?;

        sleep(Duration::from_secs(1));

        pin.set_pwm(
            Duration::from_micros(100),
            Duration::from_micros(50)
        )?;

        sleep(Duration::from_secs(1));

    }
}
