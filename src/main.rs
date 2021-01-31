use chrono::NaiveTime;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{fonts::Font24x32, prelude::*};
use embedded_graphics::{
    fonts::Text,
    pixelcolor::Rgb565,
    style::{PrimitiveStyle, TextStyle},
    *,
};
use fonts::{Font12x16, Font6x12, Font6x8};
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


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpsData {
    pub time: Option<NaiveTime>,
    pub fix_satellites: Option<u32>,
    pub pdop: Option<f32>,
    pub hdop: Option<f32>,
    pub vdop: Option<f32>,
    pub longitude: Option<f64>,
    pub latitude: Option<f64>,
    pub altitude: Option<f32>,
}

const FONT_SMALL : Font6x8 = Font6x8;
const FONT_MEDIUM : Font12x16 = Font12x16;
const FONT_LARGE : Font24x32 = Font24x32;

const DARK_GREY: Rgb565 = Rgb565::new(6, 12, 6);
const GREY: Rgb565 = Rgb565::new(12, 24, 12);
const LIGHT_GREY: Rgb565 = Rgb565::new(24, 48, 24);

impl GpsData {
    fn new() -> Self {
        GpsData {
            time: None,
            fix_satellites: None,
            pdop: None,
            hdop: None,
            vdop: None,
            longitude: None,
            latitude: None,
            altitude: None,
        }
    }
}

pub trait DisplaysGpsData {
    fn update_gps_data(&mut self, data: &GpsData);
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
        let mut ui = Ui {
            display: display,
            prev_gps_data: GpsData::new(),
        };
        ui.draw_data_labels();
        ui
    }

    fn draw_data_labels(&mut self) {
        let font = FONT_SMALL;
        let colour = GREY;
        let labels = vec![
            "# Sat:",
            "PDOP:",
            "HDOP:",
            "VDOP:",
            "Lat:",
            "Lon:",
            "Alt:",
        ];
        let start_y = 50;
        for (index, label) in labels.iter().enumerate() {
            self.draw_text_at(label, Point::new(0, start_y + (index as i32) * 20), font, colour);
        }
    }

    fn draw_text_at<F: Font + Copy>(
        &mut self,
        text: &str,
        location: Point,
        font: F,
        colour: Rgb565,
    ) {
        let t = Text::new(text, location).into_styled(TextStyle::new(font, colour));
        t.draw(&mut self.display).unwrap();
    }

    fn update_time(&mut self, opt_time: Option<NaiveTime>) {
        let location = Point::new(0, 0);
        let blank = Rectangle::new(location, location + Point::new(150, 13))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK));
        blank.draw(&mut self.display).unwrap();
        if let Some(time) = opt_time {
            self.draw_text_at(time.to_string().as_str(), location, Font12x16, DARK_GREY);
        }
    }

    fn update_satellite_count(&mut self, opt_sat_count: Option<u32>, have_fix: bool) {
        let location = Point::new(60, 28);
        let blank = Rectangle::new(location, location + Point::new(40, 26))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK));
        blank.draw(&mut self.display).unwrap_or_default();
        if let Some(sat_count) = opt_sat_count {
            let colour = match have_fix {
                true => Rgb565::new(0, 255, 0),
                false => LIGHT_GREY,
            };
            self.draw_text_at(
                sat_count.to_string().as_str(),
                location,
                FONT_LARGE,
                colour,
            );
        }
    }

    fn update_dop(&mut self, opt_dop: Option<f32>, y: i32) {
        let location = Point::new(60, y);
        let blank = Rectangle::new(location, location + Point::new(95, 13))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK));
        blank.draw(&mut self.display).unwrap();
        if let Some(dop) = opt_dop {
            self.draw_text_at(dop.to_string().as_str(), location, FONT_MEDIUM, LIGHT_GREY);
        }
    }

    fn update_latitude(&mut self, opt_latitude: Option<f64>) {
        let location = Point::new(60, 124);
        let blank = Rectangle::new(location, location + Point::new(180, 13))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK));
        blank.draw(&mut self.display).unwrap();
        if let Some(latitude) = opt_latitude {
            self.draw_text_at(
                latitude.to_string().as_str(),
                location,
                FONT_MEDIUM,
                Rgb565::new(0, 255, 0),
            );
        }
    }

    fn update_longitude(&mut self, opt_longitude: Option<f64>) {
        let location = Point::new(60, 144);
        let blank = Rectangle::new(location, location + Point::new(180, 13))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK));
        blank.draw(&mut self.display).unwrap();
        if let Some(longitude) = opt_longitude {
            self.draw_text_at(
                longitude.to_string().as_str(),
                location,
                FONT_MEDIUM,
                Rgb565::new(0, 64, 0),
            );
        }
    }

    fn update_altitude(&mut self, opt_altitude: Option<f32>) {
        let location = Point::new(60, 164);
        let blank = Rectangle::new(location, location + Point::new(95, 13))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK));
        blank.draw(&mut self.display).unwrap();
        if let Some(altitude) = opt_altitude {
            self.draw_text_at(
                altitude.to_string().as_str(),
                location,
                FONT_MEDIUM,
                Rgb565::new(0, 16, 0),
            );
        }
    }
}

impl<DT> DisplaysGpsData for Ui<DT>
where
    DT: DrawTarget<Rgb565>,
    DT::Error: core::fmt::Debug,
{
    fn update_gps_data(&mut self, data: &GpsData) {
        if data.time != self.prev_gps_data.time {
            self.update_time(data.time);
            self.prev_gps_data.time = data.time;
        }
        if data.fix_satellites != self.prev_gps_data.fix_satellites {
            self.update_satellite_count(data.fix_satellites, data.latitude.is_some());
            self.prev_gps_data.fix_satellites = data.fix_satellites;
        }
        if data.pdop != self.prev_gps_data.pdop {
            self.update_dop(data.pdop, 64);
            self.prev_gps_data.pdop = data.pdop;
        }
        if data.hdop != self.prev_gps_data.hdop {
            self.update_dop(data.hdop, 84);
            self.prev_gps_data.hdop = data.hdop;
        }
        if data.vdop != self.prev_gps_data.vdop {
            self.update_dop(data.vdop, 104);
            self.prev_gps_data.vdop = data.vdop;
        }

        if data.latitude != self.prev_gps_data.latitude {
            self.update_latitude(data.latitude);
            self.prev_gps_data.latitude = data.latitude;
        }
        if data.longitude != self.prev_gps_data.longitude {
            self.update_longitude(data.longitude);
            self.prev_gps_data.longitude = data.longitude;
        }
        if data.altitude != self.prev_gps_data.altitude {
            self.update_altitude(data.altitude);
            self.prev_gps_data.altitude = data.altitude;
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

    let uart_config = serial::PortSettings {
        baud_rate: serial::Baud9600,
        char_size: serial::Bits8,
        flow_control: serial::FlowNone,
        parity: serial::ParityNone,
        stop_bits: serial::Stop1,
    };
    let mut sp = serial::open("/dev/ttyS0").unwrap();
    println!("Opened serial port");
    sp.configure(&uart_config).unwrap();

    let mut gps_data = GpsData::new();
    let br = BufReader::new(sp);
    for line in br.lines() {
        match line {
            Ok(line) => {
                process_line(&line, &mut gps_data);
                ui.update_gps_data(&gps_data);
            }
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
                gps_info.latitude = data.lat;
                gps_info.longitude = data.lon;
            }
            nmea::ParseResult::GGA(data) => {
                println!("{:?}", data);
                gps_info.time = data.fix_time;
                gps_info.fix_satellites = data.fix_satellites;
                gps_info.latitude = data.latitude;
                gps_info.longitude = data.longitude;
                gps_info.altitude = data.altitude;
                gps_info.hdop = data.hdop;
            }
            nmea::ParseResult::GSV(data) => println!(
                "{}/{}   {:?}",
                data.sentence_num, data.number_of_sentences, data.sats_info
            ),
            nmea::ParseResult::GSA(data) => {
                println!("{:?}", data);
                gps_info.pdop = data.pdop;
                gps_info.hdop = data.hdop;
                gps_info.vdop = data.vdop;
            }
            nmea::ParseResult::VTG(data) => println!("{:?}", data),
            nmea::ParseResult::Unsupported(_) => println!("Unsupported sentence: {}", line),
            nmea::ParseResult::TXT(data) => println!("{:?}", data),
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
