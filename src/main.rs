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

    
    let mut sp = serial::open("/dev/ttyS0").unwrap();
    println!("Opened serial port");
    sp.configure(&UART_CONFIG).unwrap();
    let br = BufReader::new(sp);
    for line in br.lines() {
        match line {
            Ok(line) => process_line(&line, &mut display),
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {},
            Err(e) => println!("{:?}", e)
        }
    }
    
    Ok(())
    
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
fn process_line<D: DrawTarget<Rgb565>>(line: &str, display: &mut D) 
where
     D::Error : std::fmt::Debug
{
    match nmea::parse(line.as_bytes()) {
        Ok(r) => match r {
            nmea::ParseResult::GLL(data) => println!("{:?}", data),
            nmea::ParseResult::RMC(data) => {
                println!("{:?}", data);
                update_displayed_time(display, data.fix_time);
            },
            nmea::ParseResult::GGA(data) => {
                println!("{:?}", data);
                update_displayed_time(display, data.fix_time);
                update_displayed_satellite_count(display, data.fix_satellites);
            }
            nmea::ParseResult::GSV(data) => println!("{}/{}   {:?}", data.sentence_num, data.number_of_sentences, data.sats_info),
            nmea::ParseResult::GSA(data) => println!("{:?}", data),
            nmea::ParseResult::VTG(data) => println!("{:?}", data),
            nmea::ParseResult::TXT(data) => println!("{:?}", data),
            nmea::ParseResult::Unsupported(_) => println!("Unsupported sentence: {}", line),
            
        } 
        Err(e) => println!("Couldn't parse this nmea sentence \"{}\"\t Reason: \t\"{}\"", line, e)
    }
}

fn draw_text_at<D: DrawTarget<C>, F: Font + Copy, C: PixelColor>(display: &mut D, text: &str, location: Point, font: F, colour: C) 
where
     D::Error : std::fmt::Debug
{
    let t = Text::new(text, location)
         .into_styled(TextStyle::new(font, colour));
    t.draw(display).unwrap();
}

fn update_displayed_time<D: DrawTarget<Rgb565>>(display: &mut D, opt_time: Option<NaiveTime>)
where
     D::Error : std::fmt::Debug
{
    let location = Point::new(0,0);
    let blank = Rectangle::new(location, location + Point::new(95,13))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK));
    blank.draw(display).unwrap();
    if let Some(time) = opt_time { 
        draw_text_at(display, time.to_string().as_str(), location, Font12x16, Rgb565::new(8, 16, 8));
    }
}

fn update_displayed_satellite_count<D: DrawTarget<Rgb565>>(display: &mut D, opt_sat_count: Option<u32>) 
where
     D::Error : std::fmt::Debug
{
    let location = Point::new(10,26);
    let blank = Rectangle::new(location, location + Point::new(40,26))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK));
    blank.draw(display).unwrap_or_default();
    if let Some(sat_count) = opt_sat_count { 
        let colour = match sat_count {
            _ => Rgb565::new(255,0, 0)
        };
        draw_text_at(display, sat_count.to_string().as_str(), location, Font24x32, colour);
    }
}


fn main() {
    let result = run();
    // need to do some error handling here
    match result {
        Err(e) => println!("FATAL ERROR: {:?}", e),
        _ => ()
    }  
}
