use fonts::{Font12x16, Font8x16};
use primitives::Rectangle;
use rppal::{gpio::Gpio, spi::{Bus, Mode, SlaveSelect, Spi}, hal::Delay};
use serial::SerialPort;
use st7789::ST7789;
use core::fmt;
use std::{error::Error, fmt::Debug, io::{BufRead, BufReader}};
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{fonts::{Font24x32, Text}, *, pixelcolor::Rgb565, primitives::Circle, style::{PrimitiveStyle, TextStyle}};
use embedded_graphics::prelude::*;
use embedded_hal::blocking::delay::{DelayMs};
use chrono::NaiveTime;

const UART_CONFIG: serial::PortSettings = serial::PortSettings {
    baud_rate:          serial::Baud9600,
    char_size:          serial::Bits8,
    flow_control:       serial::FlowNone,
    parity:             serial::ParityNone,
    stop_bits:          serial::Stop1
};

enum GpsData {
    Time(Option<NaiveTime>),
    SatelliteCount(u32),
    Longitude(Option<f64>),
    Latitude(Option<f64>),
    Altitude(Option<f64>),
    Pdop(Option<f64>),
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
    display.set_orientation(st7789::Orientation::Landscape).unwrap();
    display.clear(Rgb565::BLACK).unwrap();


    let mut counter: u64 = 0;
    // let base_rectangle = Rectangle::new(Point::new(0,0), Point::new(240,240));
    loop {
        let colour = match counter % 5 {
            0 => Rgb565::BLACK,
            1 => Rgb565::MAGENTA,
            2 => Rgb565::BLUE,
            3 => Rgb565::CYAN,
            4 => Rgb565::WHITE,
            _ => Rgb565::RED,
        };
        
        display.clear(colour).unwrap();
        // base_rectangle
        //     .into_styled(PrimitiveStyle::with_fill(colour))
        //     .draw(&mut display).unwrap();
        
        counter += 1;
        delay.delay_ms(1u32);
    } 

    
    // Draw the graphics
    // let c = Circle::new(Point::new(60, 60), 14).into_styled(PrimitiveStyle::with_fill(Rgb565::RED));
    // let t = Text::new("Hello Rust!", Point::new(20, 16))
    //     .into_styled(TextStyle::new(fonts::Font12x16, Rgb565::YELLOW));
    // c.draw(&mut display).unwrap();
    // t.draw(&mut display).unwrap();

    // let mut counter = 0;
    // let b = Rectangle::new(Point::new(20,100), Point::new(70,110))
    //     .into_styled(PrimitiveStyle::with_fill(Rgb565::BLUE));
    // loop {
    //     let counter_string = format!("{}", counter);
    //     counter += 1;
    //     let t = Text::new(counter_string.as_str(), Point::new(20, 100))
    //         .into_styled(TextStyle::new(fonts::Font6x8, Rgb565::WHITE));
    //     //delay.delay_ms(10u32);
    //     b.draw(&mut display).unwrap();
    //     t.draw(&mut display).unwrap();
    // }
}



fn main() {
    let result = run();
    // need to do some error handling here
    match result {
        Err(e) => println!("FATAL ERROR: {:?}", e),
        _ => ()
    }  
}
