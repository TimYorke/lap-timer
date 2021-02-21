use embedded_graphics::{*, fonts::Font12x16, fonts::Text, pixelcolor::Rgb565, prelude::*, style::TextStyle};
use bno055::mint;
use st7789::ST7789;
use display_interface_spi::SPIInterfaceNoCS;
use rppal::{gpio::OutputPin, spi::Spi};

mod display_buffering;
use display_buffering::DisplayBuffer;

const FONT_SMALL : fonts::Font6x8 = fonts::Font6x8;
const FONT_MEDIUM : Font12x16 = Font12x16;
//const FONT_LARGE : Font24x32 = Font24x32;

const GREY: Rgb565 = Rgb565::new(12, 24, 12);

pub struct Ui {
    display: ST7789<SPIInterfaceNoCS<Spi, OutputPin>, OutputPin>,
}

impl Ui {
    pub fn new(mut display: ST7789<SPIInterfaceNoCS<Spi, OutputPin>, OutputPin>) -> Self {
        display.clear(Rgb565::BLACK).unwrap();
        let mut ui = Ui{ display };
        ui.draw_data_labels();
        ui
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
            Ui::draw_text_at(label, Point::new(0, start_y + (index as i32) * 20), font, colour, &mut self.display);
        }
    }

    pub fn display_quaternion(&mut self, q: mint::Quaternion<f32>) {
        let x = 40;
        let font = FONT_MEDIUM;
        let colour = Rgb565::new(Rgb565::MAX_R/2,Rgb565::MAX_G,Rgb565::MAX_B / 4);
        //let location = Point::new(x, 50);
        let mut buffer = DisplayBuffer::new(75, 75, Point::new(40, 50));
        Ui::draw_text_at(format!("{:+.3}",q.v.x).as_str(), Point::new(0, 0), font, colour,&mut buffer);
        Ui::draw_text_at(format!("{:+.3}",q.v.y).as_str(), Point::new(0, 20), font, colour, &mut buffer);
        Ui::draw_text_at(format!("{:+.3}",q.v.z).as_str(), Point::new(0, 40), font, colour, &mut buffer);
        Ui::draw_text_at(format!("{:+.3}",q.s).as_str(), Point::new(0, 60), font, colour, &mut buffer); 
        buffer.draw(&mut self.display).unwrap();
    }

    pub fn display_fps(&mut self, fps: u32) {
        let mut buffer = DisplayBuffer::new(50, 12, Point::new(190, 0));
        Ui::draw_text_at(format!("{} fps",fps).as_str(), Point::new(0, 0), FONT_SMALL, GREY,&mut buffer);
        buffer.draw(&mut self.display).unwrap();
    }
}