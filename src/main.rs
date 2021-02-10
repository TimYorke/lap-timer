use bno055::{BNO055OperationMode, mint};
use chrono::NaiveTime;
use crossterm::cursor;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{fonts::Font24x32, prelude::*};
use embedded_graphics::{
    fonts::Text,
    pixelcolor::Rgb565,
    style::{PrimitiveStyle, TextStyle},
    *,
};
use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use fonts::{Font12x16, Font6x8};
use primitives::Rectangle;
use rppal::{gpio::Gpio, hal::Delay, i2c::I2c, spi::{Bus, Mode, SlaveSelect, Spi}};
use st7789::ST7789;
use std::{error::Error, io::stdout};


fn main() {
    let result = run();
    // need to do some error handling here
    match result {
        Err(e) => println!("FATAL ERROR: {:?}", e),
        _ => (),
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    // SPI driver init
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 96_000_000, Mode::Mode3).unwrap();
    let pin_reset = Gpio::new()?.get(27)?.into_output();
    let pin_dc = Gpio::new()?.get(22)?.into_output();
    let spi_interface = SPIInterfaceNoCS::new(spi, pin_dc);

    // Display driver init
    let mut display = ST7789::new(spi_interface, pin_reset, 240, 240);
    let mut delay = Delay::new();
    display.init(&mut delay).unwrap();
    display
        .set_orientation(st7789::Orientation::Landscape)
        .unwrap();

    let i2c = I2c::new().unwrap();
    let mut imu = bno055::Bno055::new(i2c).with_alternative_address();
    imu.init(&mut delay).unwrap();
    imu.set_mode(bno055::BNO055OperationMode::IMU, &mut delay).unwrap();

    println!("BNO055 output:");
    loop {
        let quat: mint::Quaternion<f32> = imu.quaternion().unwrap();
        print!("\r{:?}        ", quat);
        delay.delay_ms(2u32); 
    }

    //Ok(())
}


