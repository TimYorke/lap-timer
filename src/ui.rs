use embedded_graphics::{*, fonts::Font12x16, fonts::Text, pixelcolor::Rgb565, prelude::*, primitives::Rectangle, style::{PrimitiveStyle, TextStyle}};
use bno055::mint;
use st7789::ST7789;
use display_interface_spi::SPIInterfaceNoCS;
use rppal::{gpio::OutputPin, spi::Spi};

mod display_buffering;
use display_buffering::DeltaDisplayBuffer;

const FONT_SMALL : fonts::Font6x8 = fonts::Font6x8;
const FONT_MEDIUM : Font12x16 = Font12x16;
//const FONT_LARGE : Font24x32 = Font24x32;

const GREY: Rgb565 = Rgb565::new(12, 24, 12);

pub struct Ui {
    display: ST7789<SPIInterfaceNoCS<Spi, OutputPin>, OutputPin>,
    display_buffer: DeltaDisplayBuffer,
}

impl Ui {
    pub fn new(mut display: ST7789<SPIInterfaceNoCS<Spi, OutputPin>, OutputPin>) -> Self {
        display.clear(Rgb565::BLACK).unwrap();
        let display_buffer = DeltaDisplayBuffer::new(240, 240, Point::new(0, 0));
        let mut ui = Ui{ display, display_buffer };
        ui.draw_data_labels();
        ui
    }
    pub fn flush(&mut self) {
        self.display_buffer.draw(&mut self.display).unwrap();
    }
    
    fn draw_text_at<F: Font + Copy, D: DrawTarget<Rgb565>>(
        text: &str,
        location: Point,
        font: F,
        colour: Rgb565,
        target: &mut D
    )
    where D::Error: std::fmt::Debug 
    {
        let t = Text::new(text, location).into_styled(TextStyle::new(font, colour));
        t.draw(target).unwrap();
    }

    fn draw_data_labels(&mut self) {
        let font = FONT_MEDIUM;
        let colour = GREY;
        let labels = vec![
            "x:",
            "y:",
            "z:",
            "s:",
        ];
        let start_y = 50;
        for (index, label) in labels.iter().enumerate() {
            Ui::draw_text_at(label, Point::new(0, start_y + (index as i32) * 20), font, colour, &mut self.display_buffer);
        }
    }

    pub fn draw_quaternion(&mut self, q: mint::Quaternion<f32>) {
        let x = 40;
        let font = FONT_MEDIUM;
        let colour = Rgb565::new(Rgb565::MAX_R/2,Rgb565::MAX_G,Rgb565::MAX_B / 4);
        let location = Point::new(x, 50);
        let blank = Rectangle::new(location, location + Point::new(75, 75))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK));
        blank.draw(&mut self.display_buffer).unwrap();
        Ui::draw_text_at(format!("{:+.3}",q.v.x).as_str(), Point::new(x, 50), font, colour,&mut self.display_buffer);
        Ui::draw_text_at(format!("{:+.3}",q.v.y).as_str(), Point::new(x, 70), font, colour, &mut self.display_buffer);
        Ui::draw_text_at(format!("{:+.3}",q.v.z).as_str(), Point::new(x, 90), font, colour, &mut self.display_buffer);
        Ui::draw_text_at(format!("{:+.3}",q.s).as_str(), Point::new(x, 110), font, colour, &mut self.display_buffer); 
    }

    pub fn draw_fps(&mut self, fps: u32) {
        let location = Point::new(190, 0);
        let blank = Rectangle::new(location, location + Point::new(50, 12))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK));
        blank.draw(&mut self.display_buffer).unwrap();
        Ui::draw_text_at(format!("{} fps",fps).as_str(), Point::new(190, 0), FONT_SMALL, GREY,&mut self.display_buffer);
    }
}