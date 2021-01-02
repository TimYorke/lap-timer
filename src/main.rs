use rppal::gpio::Gpio;
use std::{error::Error, thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    let mut pin = Gpio::new()?.get(20)?.into_output();

    loop {
        pin.set_high();
        sleep(Duration::from_millis(10));
        pin.set_low();
        sleep(Duration::from_millis(90));
    }
}
