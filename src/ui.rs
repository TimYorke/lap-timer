use embedded_graphics::{
    prelude::*,
    fonts::Text,
    pixelcolor::Rgb565,
    style::{PrimitiveStyle, TextStyle},
    fonts::{Font12x16},
    primitives::Rectangle,
};
use bno055::mint;
use st7789::ST7789;
use display_interface_spi::SPIInterfaceNoCS;
use rppal::{gpio::OutputPin, spi::Spi};

//const FONT_SMALL : Font6x8 = Font6x8;
const FONT_MEDIUM : Font12x16 = Font12x16;
//const FONT_LARGE : Font24x32 = Font24x32;

//const DARK_GREY: Rgb565 = Rgb565::new(6, 12, 6);
const GREY: Rgb565 = Rgb565::new(12, 24, 12);
const LIGHT_GREY: Rgb565 = Rgb565::new(24, 48, 24);

pub struct Ui {
    display: ST7789<SPIInterfaceNoCS<Spi, OutputPin>, OutputPin>
}

impl Ui {
    pub fn new(mut display: ST7789<SPIInterfaceNoCS<Spi, OutputPin>, OutputPin>) -> Self {
        display.clear(Rgb565::BLACK).unwrap();
        let mut ui = Ui{display};
        ui.draw_data_labels();
        ui
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
            self.draw_text_at(label, Point::new(0, start_y + (index as i32) * 20), font, colour);
        }
    }

    pub fn display_quaternion(&mut self, q: mint::Quaternion<f32>) {
        let x = 40;
        let font = FONT_MEDIUM;
        let colour = LIGHT_GREY;
        let location = Point::new(x, 50);
        let blank = Rectangle::new(location, location + Point::new(170, 80))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK));
        blank.draw(&mut self.display).unwrap();
        self.draw_text_at(q.v.x.to_string().as_str(), Point::new(x, 50), font, colour);
        self.draw_text_at(q.v.y.to_string().as_str(), Point::new(x, 70), font, colour);
        self.draw_text_at(q.v.z.to_string().as_str(), Point::new(x, 90), font, colour);
        self.draw_text_at(q.s.to_string().as_str(), Point::new(x, 110), font, colour); 
    }
}