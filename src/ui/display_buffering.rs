

use embedded_graphics::{pixelcolor::{Rgb565, raw::RawU16}, prelude::*, primitives::Rectangle};
use itertools::Itertools;

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
        let pixel_iter = self.buffer.buffer.iter().enumerate()
            .map(|(index, v)| -> Pixel<Rgb565> {
                let y = (index / self.buffer.width) as i32;
                let x = (index % self.buffer.width) as i32;
                Pixel(Point::new(x, y) + self.position, Rgb565::from(RawU16::new(v.clone())))
            });
        display.draw_iter(pixel_iter)?;
        Ok(()) 
    }
}

// This was much too slow because it employs single pixel writes to draw each of the changed pixels. This turns out to be around twice as slow for the case of changing text.
// An alternative would be to figure out the smallest area that covers all the changed pixels and send that down as a block. 
pub struct DeltaDisplayBuffer {
    last_buffer: Option<RawBuffer>,
    current_buffer: RawBuffer,
    changed_map: Vec<Point>,
    position: Point,
}

impl DeltaDisplayBuffer {
    pub fn new(width: usize, height: usize, position: Point) -> Self {
        DeltaDisplayBuffer { 
            last_buffer: None, 
            current_buffer: RawBuffer::new(width, height), 
            changed_map: Vec::new(),
            position 
        }
    }

    pub fn draw<D: DrawTarget<Rgb565>>(&mut self, display: &mut D) -> Result<(), D::Error> {
        match self.last_buffer {
            Some(ref lb) => {
                let pixel_iter = self.changed_map.iter()
                .map(|p| {
                    let index = self.current_buffer.width * p.y as usize + p.x as usize;
                    (p, index)
                })
                .filter(|(_, index)| self.current_buffer.buffer[index.clone()] != lb.buffer[index.clone()]) 
                .map(|(p, index)| -> Pixel<Rgb565> {
                    Pixel(p.clone(), Rgb565::from(RawU16::new(self.current_buffer.buffer[index])))
                });
                display.draw_iter(pixel_iter)?;
            },
            None => {
                let pixel_iter = self.current_buffer.buffer.iter().enumerate()
                .map(|(index, v)| -> Pixel<Rgb565> {
                    let y = (index / self.current_buffer.width) as i32;
                    let x = (index % self.current_buffer.width) as i32;
                    Pixel(Point::new(x, y) + self.position, Rgb565::from(RawU16::new(v.clone())))
                });
                display.draw_iter(pixel_iter)?;
            }
        };
        self.changed_map.clear();
        self.last_buffer = Some(self.current_buffer.clone());
        Ok(())
    }
}


impl DrawTarget<Rgb565> for DeltaDisplayBuffer {
    type Error = u32;

    fn draw_pixel(&mut self, item: Pixel<Rgb565>) -> Result<(), Self::Error> {
        let colour : u16 = item.1.into_storage();
        let index = item.0.y as usize * self.current_buffer.width + item.0.x as usize;
        if let Some(ref lb) = self.last_buffer {
            self.current_buffer.buffer[index] = colour;
            if colour != lb.buffer[index] {
                self.changed_map.push(item.0); 
            } else {
            }
        } else {
            self.current_buffer.buffer[index] = colour;
        }
        Ok(())
    }

    fn size(&self) -> Size {
        self.current_buffer.size()
    }

    fn clear(&mut self, color: Rgb565) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        Rectangle::new(Point::zero(), Point::new(-1, -1) + self.size())
            .into_styled(embedded_graphics::style::PrimitiveStyle::with_fill(color))
            .draw(self)
    }
}
