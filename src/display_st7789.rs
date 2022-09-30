use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::pixelcolor::Rgb565;

use anyhow::Result;
use esp_idf_hal::gpio::*;
use esp_idf_sys::EspError;
use st7789::Orientation;

const TFT_WIDTH: u16 = 240;
const TFT_HEIGHT: u16 = 135;
const TFT_X_OFFSET: u16 = 40;
const TFT_Y_OFFSET: u16 = 53;

pub struct Display<SPI, DC, RST>
where
  SPI: embedded_hal_0_2::blocking::spi::Write<u8>,
  DC: embedded_hal_0_2::digital::v2::OutputPin,
  RST: embedded_hal_0_2::digital::v2::OutputPin,
{
  deriver: st7789::ST7789<SPIInterfaceNoCS<SPI, DC>, RST, Gpio0<Output>>,
}

impl<SPI, DC, RST> Display<SPI, DC, RST>
where
  SPI: embedded_hal_0_2::blocking::spi::Write<u8>,
  DC: embedded_hal_0_2::digital::v2::OutputPin,
  RST: embedded_hal_0_2::digital::v2::OutputPin<Error = EspError>,
{
  pub fn new(spi: SPI, tft_dc: DC, tft_rst: RST) -> Result<Self, ()> {
    let tft_width = (TFT_WIDTH + TFT_X_OFFSET) as u16;
    let tft_height = (TFT_HEIGHT + TFT_Y_OFFSET) as u16;

    let di = SPIInterfaceNoCS::new(spi, tft_dc);
    let mut display = st7789::ST7789::new(di, Some(tft_rst), None, tft_width, tft_height);

    let mut delay = esp_idf_hal::delay::Ets {};
    display.init(&mut delay).map_err(|_| ())?;
    display
      .set_orientation(Orientation::Landscape)
      .map_err(|_| ())?;

    return Ok(Display { deriver: display });
  }

  pub fn width(&mut self) -> usize {
    TFT_WIDTH as usize
  }

  pub fn height(&mut self) -> usize {
    TFT_HEIGHT as usize
  }

  pub fn draw(
    &mut self,
    display_buffer: &mut super::display_buffer::DisplayBuffer<Rgb565>,
  ) -> Result<(), ()> {
    let ex = (self.width() - 1) as u16;
    let ey = (self.height() - 1) as u16;
    let data = display_buffer.as_pixels();

    self
      .deriver
      .set_pixels(
        TFT_X_OFFSET,
        TFT_Y_OFFSET,
        ex + TFT_X_OFFSET,
        ey + TFT_Y_OFFSET,
        data.iter().map(|e| *e),
      )
      .map_err(|_| ())?;
    Ok(())
  }
}
