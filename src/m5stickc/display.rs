use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

use embedded_graphics::primitives::Rectangle;
use st7735_lcd::Orientation;

use anyhow::Result;

const TFT_WIDTH: i32 = 160;
const TFT_HEIGHT: i32 = 80;
const TFT_X_OFFSET: i32 = 1;
const TFT_Y_OFFSET: i32 = 26;

pub struct Display<SPI, DC, RST>
where
  SPI: embedded_hal_0_2::blocking::spi::Write<u8>,
  DC: embedded_hal_0_2::digital::v2::OutputPin,
  RST: embedded_hal_0_2::digital::v2::OutputPin,
{
  deriver: st7735_lcd::ST7735<SPI, DC, RST>,
}

impl<SPI, DC, RST> Display<SPI, DC, RST>
where
  SPI: embedded_hal_0_2::blocking::spi::Write<u8>,
  DC: embedded_hal_0_2::digital::v2::OutputPin,
  RST: embedded_hal_0_2::digital::v2::OutputPin,
{
  pub fn new(spi: SPI, tft_dc: DC, tft_rst: RST) -> Result<Self, ()> {
    let tft_width = (TFT_WIDTH + TFT_X_OFFSET) as u32;
    let tft_height = (TFT_HEIGHT + TFT_Y_OFFSET) as u32;

    let mut display =
      st7735_lcd::ST7735::new(spi, tft_dc, tft_rst, true, false, tft_width, tft_height);
    let mut delay = esp_idf_hal::delay::Ets {};
    display.init(&mut delay)?;
    display.set_offset(TFT_X_OFFSET as u16, TFT_Y_OFFSET as u16);
    display.set_orientation(&Orientation::Landscape)?;

    return Ok(Display {
      deriver: display,
    });
  }

  pub fn width(&mut self) -> usize {
    TFT_WIDTH as usize
  }

  pub fn height(&mut self) -> usize {
    TFT_HEIGHT as usize
  }

  pub fn clear(&mut self) -> Result<(), ()> {
    self.deriver.clear(Rgb565::WHITE)
  }

  pub fn draw(&mut self, display_buffer: &super::display_buffer::DisplayBuffer<Rgb565>) -> Result<(), ()> {
    let area = Rectangle::new(Point::new(0, 0), Size::new(self.width() as u32, self.height() as u32));
    let iter = display_buffer.buffer.iter().map(|e| *e);

    self.deriver.fill_contiguous(&area, iter)
  }
}
