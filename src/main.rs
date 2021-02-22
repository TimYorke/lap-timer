mod ui;
use ui::*;
mod fps_monitor;
use fps_monitor::*;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use rppal::{gpio::Gpio, hal::Delay, i2c::I2c, spi::{Bus, Mode, SlaveSelect, Spi}};
use st7789::ST7789;
use std::error::Error;


fn main() {
    let mut delay = Delay::new();
    let mut ui = create_ui(&mut delay).unwrap();
    let mut imu = create_imu(&mut delay);
    let mut fps_mon = FpsMonitor::start_new(200);
    loop {
        ui.display_quaternion(imu.quaternion().unwrap());
        
        fps_mon.on_frame();
        if let Some(fps) = fps_mon.get_fps() {
            ui.display_fps(fps);
        }

        delay.delay_ms(5u32);
    }
}

fn create_imu(delay: &mut Delay) -> bno055::Bno055<I2c>  {
    let i2c = I2c::new().unwrap();
    let mut imu = bno055::Bno055::new(i2c).with_alternative_address();
    imu.init(delay).unwrap();
    imu.set_mode(bno055::BNO055OperationMode::NDOF, delay).unwrap();
    imu
}

fn create_ui(delay: &mut Delay) -> Result<ui::Ui, Box<dyn Error>> {
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 96_000_000, Mode::Mode3).unwrap();
    let pin_reset = Gpio::new()?.get(27)?.into_output();
    let pin_dc = Gpio::new()?.get(22)?.into_output();
    let spi_interface = SPIInterfaceNoCS::new(spi, pin_dc);
    let mut display = ST7789::new(spi_interface, pin_reset, 240, 240);
    display.init(delay).unwrap();
    display.set_orientation(st7789::Orientation::Landscape).unwrap();
    Ok(Ui::new(display))
}
