use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use esp_idf_hal::gpio::*;

use st7735_lcd::Orientation;

use anyhow::Result;

type Spi3Master = esp_idf_hal::spi::Master<
  esp_idf_hal::spi::SPI3,
  Gpio13<Unknown>,
  Gpio15<Unknown>,
  Gpio14<Unknown>,
  Gpio5<Unknown>,
>;
type Driver = st7735_lcd::ST7735<Spi3Master, Gpio23<Output>, Gpio18<Output>>;

const TFT_WIDTH: i32 = 160;
const TFT_HEIGHT: i32 = 80;
const TFT_X_OFFSET: i32 = 1;
const TFT_Y_OFFSET: i32 = 26;

pub struct Display {
  deriver: Driver,
  cursur: Point,
  text_font: &'static MonoFont<'static>,
}

impl Display {
  pub fn new(spi: Spi3Master, tft_dc: Gpio23<Output>, tft_rst: Gpio18<Output>) -> Result<Self, ()> {
    let tft_width = (TFT_WIDTH + TFT_X_OFFSET) as u32;
    let tft_height = (TFT_HEIGHT + TFT_Y_OFFSET) as u32;

    let mut display =
      st7735_lcd::ST7735::new(spi, tft_dc, tft_rst, true, false, tft_width, tft_height);
    let mut delay = esp_idf_hal::delay::Ets {};
    display.init(&mut delay)?;
    display.set_orientation(&Orientation::Landscape)?;
    display.clear(Rgb565::WHITE)?;

    return Ok(Display {
      deriver: display,
      cursur: Point::new(0, 0),
      text_font: &embedded_graphics::mono_font::jis_x0201::FONT_9X15,
    });
  }

  // TODO: carriage control
  pub fn print(&mut self, text: &str) -> Result<(), ()> {
    let text_style = MonoTextStyle::new(self.text_font, Rgb565::BLACK);
    let cursur = Text::with_baseline(text, self.cursur, text_style, Baseline::Top)
      .translate(Point::new(TFT_X_OFFSET, TFT_Y_OFFSET))
      .draw(&mut self.deriver)?;
    self.cursur.x = cursur.x - TFT_X_OFFSET;
    self.cursur.y = cursur.y - TFT_Y_OFFSET;
    return Ok(());
  }
}
