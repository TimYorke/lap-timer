use embedded_graphics::{image::{Image, ImageRaw, ImageRawLE}, pixelcolor::{Rgb565, raw::RawU16}, prelude::*};

#[derive(Clone)]
struct RawBuffer {
    buffer: Vec<u16>,
    width: usize,
    height: usize,   
}

impl RawBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let buffer: Vec<u16> = vec![0u16; width * height];
        RawBuffer { buffer, width, height }
    }
}

impl DrawTarget<Rgb565> for RawBuffer {
    type Error = u32;

    fn draw_pixel(&mut self, item: Pixel<Rgb565>) -> Result<(), Self::Error> {
        let colour : u16 = item.1.into_storage();
        let col = item.0.x as usize;
        let row = item.0.y as usize; 
        self.buffer[col + (row * self.width)] = colour;
        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

pub struct DisplayBuffer {
    buffer: RawBuffer,
    position: Point,
}

impl DisplayBuffer {
    pub fn new(width: usize, height: usize, position: Point) -> Self {
        let buffer = RawBuffer::new(width, height);
        DisplayBuffer { buffer, position }
    }
}

impl DrawTarget<Rgb565> for DisplayBuffer {
    type Error = u32;

    fn draw_pixel(&mut self, item: Pixel<Rgb565>) -> Result<(), Self::Error> {
        self.buffer.draw_pixel(item)
    }

    fn size(&self) -> Size {
        self.buffer.size()
    }
}

impl Drawable<Rgb565> for DisplayBuffer {
    fn draw<D: DrawTarget<Rgb565>>(self, display: &mut D) -> Result<(), D::Error> {
        let as_array: &[u16] = &self.buffer.buffer;
        let as_array8_2: &[u8] = unsafe { std::slice::from_raw_parts(as_array.as_ptr() as *mut u8, as_array.len() * 2) };
        let raw_image: ImageRawLE<Rgb565> = ImageRaw::new(
            as_array8_2,
            self.buffer.width as u32, 
            self.buffer.height as u32);
        let image: Image<_, Rgb565> = Image::new(&raw_image, self.position);
        image.draw(display)?;
        Ok(()) 
    }
}

// This was much too slow because it employs single pixel writes to draw each of the changed pixels. This turns out to be around twice as slow for the case of changing text.
// An alternative would be to figure out the smallest area that covers all the changed pixels and send that down as a block. 
pub struct DeltaDisplayBuffer {
    last_buffer: Option<RawBuffer>,
    current_buffer: RawBuffer,
    position: Point,
}

impl DeltaDisplayBuffer {
    pub fn new(width: usize, height: usize, position: Point) -> Self {
        DeltaDisplayBuffer { 
            last_buffer: None, 
            current_buffer: RawBuffer::new(width, height), 
            position 
        }
    }

    pub fn draw<D: DrawTarget<Rgb565>>(&mut self, display: &mut D) -> Result<(), D::Error> {
        match self.last_buffer {
            Some(ref mut last_buffer) => {
                for row in 0..self.current_buffer.height {
                    for col in 0..self.current_buffer.width {
                        let index = col + (row * self.current_buffer.width);
                        if self.current_buffer.buffer[index] != last_buffer.buffer[index] {
                            display.draw_pixel(Pixel(
                                Point::new(col as i32, row as i32) + self.position,
                                Rgb565::from(RawU16::new(self.current_buffer.buffer[index])))
                            )?;
                            last_buffer.buffer[index] = self.current_buffer.buffer[index];
                        }
                        self.current_buffer.buffer[index] = 0; // this should really be the default background colour
                    }
                }
            },
            None => {
                self.last_buffer = Some(self.current_buffer.clone());
                for row in 0..self.current_buffer.height {
                    for col in 0..self.current_buffer.width {
                        let index = col + (row * self.current_buffer.width);
                        display.draw_pixel(Pixel(
                            Point::new(col as i32, row as i32) + self.position,
                            Rgb565::from(RawU16::new(self.current_buffer.buffer[index])))
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl DrawTarget<Rgb565> for DeltaDisplayBuffer {
    type Error = u32;

    fn draw_pixel(&mut self, item: Pixel<Rgb565>) -> Result<(), Self::Error> {
        self.current_buffer.draw_pixel(item)
    }

    fn size(&self) -> Size {
        self.current_buffer.size()
    }
}
