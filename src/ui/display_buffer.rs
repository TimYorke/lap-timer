use embedded_graphics::{image::{Image, ImageRaw}, pixelcolor::Rgb565, prelude::*};

pub struct DisplayBuffer {
    buffer: Vec<u8>,
    width: usize,
    height: usize,
    position: Point,
}

impl DisplayBuffer {
    pub fn new(width: usize, height: usize, position: Point) -> Self {
        let buffer: Vec<u8> = vec![0u8; width * height * 2];
        DisplayBuffer { buffer, width, height, position }
    }
}

impl DrawTarget<Rgb565> for DisplayBuffer {
    type Error = u32;

    fn draw_pixel(&mut self, item: Pixel<Rgb565>) -> Result<(), Self::Error> {
        let colour : u16 = item.1.into_storage();
        let col = item.0.x as usize;
        let row = item.0.y as usize; 
        self.buffer[(col * 2) + (row * self.width * 2)] = ((colour >> 8) & 0xFF) as u8;
        self.buffer[(col * 2) + 1 + (row * self.width * 2)] = (colour & 0xFF) as u8;
        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

impl Drawable<Rgb565> for DisplayBuffer {
    fn draw<D: DrawTarget<Rgb565>>(self, display: &mut D) -> Result<(), D::Error> {
        let raw_image: ImageRaw<Rgb565> = ImageRaw::new(&self.buffer, self.width as u32, self.height as u32);
        let image: Image<_, Rgb565> = Image::new(&raw_image, self.position);
        image.draw(display)?;
        Ok(()) 
    }
}