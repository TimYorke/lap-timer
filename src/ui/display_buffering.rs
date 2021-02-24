

use embedded_graphics::{pixelcolor::{Rgb565, raw::RawU16}, prelude::*, primitives::Rectangle};

const MAX_DIRTY_PIXEL_MAP_SIZE: usize = 10000;

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

pub struct DeltaDisplayBuffer {
    last_buffer: Option<RawBuffer>,
    current_buffer: RawBuffer,
    dirty_pixel_map: Vec<Point>,
    position: Point,
}

impl DeltaDisplayBuffer {
    pub fn new(width: usize, height: usize, position: Point) -> Self {
        DeltaDisplayBuffer { 
            last_buffer: None, 
            current_buffer: RawBuffer::new(width, height), 
            dirty_pixel_map: Vec::with_capacity(MAX_DIRTY_PIXEL_MAP_SIZE),
            position 
        }
    }

    pub fn draw<D: DrawTarget<Rgb565>>(&mut self, display: &mut D) -> Result<(), D::Error> {
        match self.last_buffer {
            Some(ref lb) => {
                if self.dirty_pixel_map.len() < MAX_DIRTY_PIXEL_MAP_SIZE {
                    let pixel_iter = self.dirty_pixel_map.iter()
                    .map(|p| {
                        let index = self.current_buffer.width * p.y as usize + p.x as usize;
                        (p, index)
                    })
                    .filter(|(_, index)| self.current_buffer.buffer[index.clone()] != lb.buffer[index.clone()]) 
                    .map(|(p, index)| -> Pixel<Rgb565> {
                        Pixel(p.clone(), Rgb565::from(RawU16::new(self.current_buffer.buffer[index])))
                    });
                    display.draw_iter(pixel_iter)?;
                } else {
                    let pixel_iter = self.current_buffer.buffer.iter().enumerate()
                    .filter(|(index, v)| v.clone().clone() != lb.buffer[index.clone()])
                    .map(|(index, v)| -> Pixel<Rgb565> {
                        let p = Point::new( (index % self.current_buffer.width) as i32, (index / self.current_buffer.width) as i32);
                        Pixel(p, Rgb565::from(RawU16::new(v.clone())))
                    });
                    display.draw_iter(pixel_iter)?;
                }
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
        self.dirty_pixel_map.clear();
        self.last_buffer = Some(self.current_buffer.clone());
        Ok(())
    }
}


impl DrawTarget<Rgb565> for DeltaDisplayBuffer {
    type Error = u32;

    fn draw_pixel(&mut self, item: Pixel<Rgb565>) -> Result<(), Self::Error> {
        let new_colour : u16 = item.1.into_storage();
        let index = item.0.y as usize * self.current_buffer.width + item.0.x as usize;
        if let Some(ref lb) = self.last_buffer {
            self.current_buffer.buffer[index] = new_colour;
            let last_colour = lb.buffer[index];
            if new_colour != last_colour && self.dirty_pixel_map.len() < MAX_DIRTY_PIXEL_MAP_SIZE {
                self.dirty_pixel_map.push(item.0); 
            } else {
            }
        } else {
            self.current_buffer.buffer[index] = new_colour;
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
