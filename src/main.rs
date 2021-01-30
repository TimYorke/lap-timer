use primitives::Rectangle;
use rppal::{gpio::Gpio, spi::{Bus, Mode, SlaveSelect, Spi}, hal::Delay};
use st7789::ST7789;
use std::{error::Error};
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{fonts::{Font24x32, Text}, *, pixelcolor::Rgb565, primitives::Circle, style::{PrimitiveStyle, TextStyle}};
use embedded_graphics::prelude::*;
use embedded_hal::blocking::delay::{DelayMs};

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
    display.set_orientation(st7789::Orientation::Landscape).unwrap();
    display.clear(Rgb565::new(10, 10, 10)).unwrap();

    // Draw the graphics
    let c = Circle::new(Point::new(60, 60), 14).into_styled(PrimitiveStyle::with_fill(Rgb565::RED));
    let t = Text::new("Hello Rust!", Point::new(20, 16))
        .into_styled(TextStyle::new(fonts::Font12x16, Rgb565::YELLOW));
    c.draw(&mut display).unwrap();
    t.draw(&mut display).unwrap();

    let mut counter = 0;
    let b = Rectangle::new(Point::new(20,100), Point::new(70,110))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLUE));
    loop {
        let counter_string = format!("{}", counter);
        counter += 1;
        let t = Text::new(counter_string.as_str(), Point::new(20, 100))
            .into_styled(TextStyle::new(fonts::Font6x8, Rgb565::WHITE));
        //delay.delay_ms(10u32);
        b.draw(&mut display).unwrap();
        t.draw(&mut display).unwrap();
    }
}

fn main() {
    let result = run();
    // need to do some error handling here
    result.unwrap();    
}

