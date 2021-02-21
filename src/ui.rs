use embedded_graphics::{fonts::Font12x16, fonts::Text, image::{Image, ImageRaw, ImageRawLE}, pixelcolor::Rgb565, prelude::*, primitives::Rectangle, style::{PrimitiveStyle, TextStyle}};
use bno055::mint;
use st7789::ST7789;
use display_interface_spi::SPIInterfaceNoCS;
use rppal::{gpio::OutputPin, spi::Spi};

//const FONT_SMALL : Font6x8 = Font6x8;
const FONT_MEDIUM : Font12x16 = Font12x16;
//const FONT_LARGE : Font24x32 = Font24x32;

const GREY: Rgb565 = Rgb565::new(12, 24, 12);



#[derive(Clone)]
struct DisplayBuffer<const WIDTH: usize, const HEIGHT: usize> 
where [u8; WIDTH * HEIGHT * 2]: Sized
{
    buffer: [u8; WIDTH * HEIGHT * 2],
}

impl<const WIDTH: usize, const HEIGHT: usize> DrawTarget<Rgb565> for DisplayBuffer<WIDTH, HEIGHT> 
where [u8; WIDTH * HEIGHT * 2]: Sized
{
    type Error = u32;

    fn draw_pixel(&mut self, item: Pixel<Rgb565>) -> Result<(), Self::Error> {
        let colour : u16 = item.1.into_storage();
        let col = item.0.x as usize;
        let row = item.0.y as usize; 
        self.buffer[(col * 2) + (row * WIDTH)] = ((colour >> 8) & 0xFF) as u8;
        self.buffer[(col * 2) + 1 + (row * WIDTH)] = (colour & 0xFF) as u8;
        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Drawable<Rgb565> for DisplayBuffer<WIDTH, HEIGHT> 
where [u8; WIDTH * HEIGHT * 2]: Sized
{
    fn draw<D: DrawTarget<Rgb565>>(self, display: &mut D) -> Result<(), D::Error> {
        let raw_image: ImageRaw<Rgb565> = ImageRaw::new(&self.buffer, WIDTH as u32, HEIGHT as u32);
        let image: Image<_, Rgb565> = Image::new(&raw_image, Point::zero());
        image.draw(display)?;
        Ok(()) 
    }
}



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
        let mut draw_buffer: DisplayBuffer<57, 75> = DisplayBuffer{ buffer: [0; 57 * 75 * 2]};
        //let blank = Rectangle::new(Point::zero(), Point::new(57, 75))
        //    .into_styled(PrimitiveStyle::with_fill(Rgb565::RED));
        //blank.draw(&mut draw_buffer).unwrap();
        Ui::draw_text_at(format!("{:+.2}",q.v.x).as_str(), Point::new(0, 0), font, colour,&mut draw_buffer);
        Ui::draw_text_at(format!("{:+.2}",q.v.y).as_str(), Point::new(0, 20), font, colour, &mut draw_buffer);
        Ui::draw_text_at(format!("{:+.2}",q.v.z).as_str(), Point::new(0, 40), font, colour, &mut draw_buffer);
        Ui::draw_text_at(format!("{:+.2}",q.s).as_str(), Point::new(0, 60), font, colour, &mut draw_buffer); 
        draw_buffer.draw(&mut self.display).unwrap();
    }
}