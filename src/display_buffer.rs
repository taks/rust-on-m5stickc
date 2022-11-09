use alloc::{boxed::Box, vec};
use embedded_graphics::{
  mono_font::{MonoFont, MonoTextStyle},
  prelude::{DrawTarget, OriginDimensions, PixelColor, Point, Size},
  text::{renderer::TextRenderer, Baseline},
  Pixel,
};

pub struct DisplayBuffer<C: PixelColor> {
  pub buffer: Box<[C]>,
  width: usize,
  height: usize,
  pub cursur: Point,
  text_color: C,
  background_color: C,
  text_font: &'static MonoFont<'static>,
}

impl<C: PixelColor> DisplayBuffer<C> {
  pub fn new(background_color: C, text_color: C, width: usize, height: usize) -> Self {
    let buffer = vec![background_color; width * height].into_boxed_slice();
    Self {
      buffer,
      width,
      height,
      cursur: Point::new(0, 0),
      text_font: &embedded_graphics::mono_font::ascii::FONT_6X13,
      text_color,
      background_color,
    }
  }

  pub fn print(&mut self, text: &str) {
    let text_style = MonoTextStyle::new(self.text_font, self.text_color);
    for (i, line) in text.split('\n').enumerate() {
      let position = if i == 0 {
        self.cursur
      } else {
        Point::new(0, self.cursur.y + (text_style.line_height() as i32))
      };
      self.cursur = text_style
        .draw_string(line, position, Baseline::Top, self)
        .unwrap();
    }
  }

  fn point_to_index(&self, p: Point) -> usize {
    self.width * p.y as usize + p.x as usize
  }

  pub fn set_color_at(&mut self, p: Point, color: C) {
    let index = self.point_to_index(p);
    self.buffer[index] = color;
  }

  pub fn get_color_at(&self, p: Point) -> C {
    let index = self.point_to_index(p);
    self.buffer[index]
  }

  pub fn clear_default(&mut self) {
    self.buffer.fill(self.background_color);
  }

  pub fn as_pixels(&mut self) -> &[u16] {
    super::misc::as_mut_slice_of::<C, u16>(&self.buffer)
  }

  pub fn as_bytes<>(&mut self) -> &[u8] {
    super::misc::as_mut_slice_of::<C, u8>(&self.buffer)
  }
}

impl<C: PixelColor> OriginDimensions for DisplayBuffer<C> {
  fn size(&self) -> Size {
    Size::new(self.width as u32, self.height as u32)
  }
}

impl<C: PixelColor> DrawTarget for DisplayBuffer<C> {
  type Color = C;
  type Error = core::convert::Infallible;

  fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
  where
    I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
  {
    for Pixel(coord, color) in pixels.into_iter() {
      if coord.x >= 0 && coord.x < self.width as i32 && coord.y >= 0 && coord.y < self.height as i32
      {
        self.set_color_at(coord, color);
      }
    }
    Ok(())
  }

  fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
    self.buffer.fill(color);
    Ok(())
  }
}

impl<C: PixelColor> core::fmt::Write for DisplayBuffer<C> {
  fn write_str(&mut self, text: &str) -> Result<(), core::fmt::Error> {
    self.print(text);
    Ok(())
  }
}
