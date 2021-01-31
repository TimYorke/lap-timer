use chrono::NaiveTime;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{fonts::Font24x32, prelude::*};
use embedded_graphics::{
    fonts::{Text},
    pixelcolor::Rgb565,
    style::{PrimitiveStyle, TextStyle},
    *,
};
use fonts::Font12x16;
use primitives::Rectangle;
use rppal::{
    gpio::Gpio,
    hal::Delay,
    spi::{Bus, Mode, SlaveSelect, Spi},
};
use serial::SerialPort;
use st7789::ST7789;
use std::{
    error::Error,
    io::{BufRead, BufReader},
};

const UART_CONFIG: serial::PortSettings = serial::PortSettings {
    baud_rate: serial::Baud9600,
    char_size: serial::Bits8,
    flow_control: serial::FlowNone,
    parity: serial::ParityNone,
    stop_bits: serial::Stop1,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpsData {
    pub time: Option<NaiveTime>,
    pub fix_satellites: Option<u32>,
    pub pdop: Option<f64>,
    pub longitude: Option<f64>,
    pub latitude: Option<f64>,
    pub altitude: Option<f64>,
}

impl GpsData {
    fn new() -> Self {
        GpsData {
            time: None,
            fix_satellites: None,
            pdop: None,
            longitude: None,
            latitude: None,
            altitude: None,
        }
    }
}

pub trait DisplaysGpsData {
    fn display_gps_data(&mut self, data: &GpsData);
}
struct Ui<DT>
where
    DT: DrawTarget<Rgb565>,
{
    display: DT,
    prev_gps_data: GpsData,
}

impl<DT> Ui<DT>
where
    DT: DrawTarget<Rgb565>,
    DT::Error: core::fmt::Debug,
{
    fn new(display: DT) -> Self {
        Ui { 
            display: display,
            prev_gps_data: GpsData::new(),
        }
    }

    fn draw_text_at<F: Font + Copy>(&mut self, text: &str, location: Point, font: F, colour: Rgb565) {
        let t = Text::new(text, location).into_styled(TextStyle::new(font, colour));
        t.draw(&mut self.display).unwrap();
    }

    fn update_satellite_count(&mut self, opt_sat_count: Option<u32>) {
        let location = Point::new(10,26);
        let blank = Rectangle::new(location, location + Point::new(40,26))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK));
        blank.draw(&mut self.display).unwrap_or_default();
        if let Some(sat_count) = opt_sat_count {
            let colour = match sat_count {
                _ => Rgb565::new(255,0, 0)
            };
            self.draw_text_at(sat_count.to_string().as_str(), location, Font24x32, colour);
        }
    }

    fn update_time(&mut self, opt_time: Option<NaiveTime>) {
        let location = Point::new(0, 0);
        let blank = Rectangle::new(location, location + Point::new(95, 13))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK));
        blank.draw(&mut self.display).unwrap();
        if let Some(time) = opt_time {
            self.draw_text_at(
                time.to_string().as_str(),
                location,
                Font12x16,
                Rgb565::new(8, 16, 8),
            );
        }
    }
}

impl<DT> DisplaysGpsData for Ui<DT>
where
    DT: DrawTarget<Rgb565>,
    DT::Error: core::fmt::Debug,
{
    fn display_gps_data(&mut self, data: &GpsData) {
        if data.time != self.prev_gps_data.time {
            self.update_time(data.time);
            self.prev_gps_data.time = data.time;
        }
        if data.fix_satellites != self.prev_gps_data.fix_satellites {
            self.update_satellite_count(data.fix_satellites);
            self.prev_gps_data.fix_satellites = data.fix_satellites;
        }
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
    display.clear(Rgb565::BLACK).unwrap();

    let mut ui = Ui::new(display);

    let mut gps_info = GpsData::new();
    let mut sp = serial::open("/dev/ttyS0").unwrap();
    println!("Opened serial port");
    sp.configure(&UART_CONFIG).unwrap();
    let br = BufReader::new(sp);
    for line in br.lines() {
        match line {
            Ok(line) => {
                process_line(&line, &mut gps_info);
                ui.display_gps_data(&gps_info);
            },
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(())
}

fn process_line(line: &str, gps_info: &mut GpsData) {
    match nmea::parse(line.as_bytes()) {
        Ok(r) => match r {
            nmea::ParseResult::GLL(data) => println!("{:?}", data),
            nmea::ParseResult::RMC(data) => {
                println!("{:?}", data);
                gps_info.time = data.fix_time;
            }
            nmea::ParseResult::GGA(data) => {
                println!("{:?}", data);
                gps_info.time = data.fix_time;
                gps_info.fix_satellites = data.fix_satellites;
            }
            nmea::ParseResult::GSV(data) => println!(
                "{}/{}   {:?}",
                data.sentence_num, data.number_of_sentences, data.sats_info
            ),
            nmea::ParseResult::GSA(data) => println!("{:?}", data),
            nmea::ParseResult::VTG(data) => println!("{:?}", data),
            nmea::ParseResult::TXT(data) => println!("{:?}", data),
            nmea::ParseResult::Unsupported(_) => println!("Unsupported sentence: {}", line),
        },
        Err(e) => println!(
            "Couldn't parse this nmea sentence \"{}\"\t Reason: \t\"{}\"",
            line, e
        ),
    }
}

fn main() {
    let result = run();
    // need to do some error handling here
    match result {
        Err(e) => println!("FATAL ERROR: {:?}", e),
        _ => (),
    }
}
