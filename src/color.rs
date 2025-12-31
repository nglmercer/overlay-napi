//! Color management and manipulation utilities

use napi_derive::napi;

#[napi(object)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: u8,
}

impl Color {
  pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
    Self { r, g, b, a }
  }

  pub fn to_rgba(&self) -> [u8; 4] {
    [self.r, self.g, self.b, self.a]
  }

  pub fn to_hex(&self) -> String {
    format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
  }

  pub fn to_rgb_hex(&self) -> String {
    format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
  }

  /// Blends this color over another using alpha compositing
  pub fn blend(&self, other: &Color) -> Color {
    let alpha = self.a as f32 / 255.0;
    let r = (self.r as f32 * alpha + other.r as f32 * (1.0 - alpha)) as u8;
    let g = (self.g as f32 * alpha + other.g as f32 * (1.0 - alpha)) as u8;
    let b = (self.b as f32 * alpha + other.b as f32 * (1.0 - alpha)) as u8;
    let a = (self.a as f32 * alpha + other.a as f32 * (1.0 - alpha)) as u8;
    Color::new(r, g, b, a)
  }

  /// Linearly interpolates between two colors
  pub fn lerp(&self, other: &Color, t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    let r = (self.r as f64 + (other.r as f64 - self.r as f64) * t) as u8;
    let g = (self.g as f64 + (other.g as f64 - self.g as f64) * t) as u8;
    let b = (self.b as f64 + (other.b as f64 - self.b as f64) * t) as u8;
    let a = (self.a as f64 + (other.a as f64 - self.a as f64) * t) as u8;
    Color::new(r, g, b, a)
  }
}

// Common colors as constants
pub const COLOR_RED: Color = Color {
  r: 255,
  g: 0,
  b: 0,
  a: 255,
};
pub const COLOR_GREEN: Color = Color {
  r: 0,
  g: 255,
  b: 0,
  a: 255,
};
pub const COLOR_BLUE: Color = Color {
  r: 0,
  g: 0,
  b: 255,
  a: 255,
};
pub const COLOR_BLACK: Color = Color {
  r: 0,
  g: 0,
  b: 0,
  a: 255,
};
pub const COLOR_WHITE: Color = Color {
  r: 255,
  g: 255,
  b: 255,
  a: 255,
};
pub const COLOR_YELLOW: Color = Color {
  r: 255,
  g: 255,
  b: 0,
  a: 255,
};
pub const COLOR_CYAN: Color = Color {
  r: 0,
  g: 255,
  b: 255,
  a: 255,
};
pub const COLOR_MAGENTA: Color = Color {
  r: 255,
  g: 0,
  b: 255,
  a: 255,
};
pub const COLOR_GRAY: Color = Color {
  r: 128,
  g: 128,
  b: 128,
  a: 255,
};
pub const COLOR_DARK_GRAY: Color = Color {
  r: 64,
  g: 64,
  b: 64,
  a: 255,
};
pub const COLOR_LIGHT_GRAY: Color = Color {
  r: 192,
  g: 192,
  b: 192,
  a: 255,
};
pub const COLOR_ORANGE: Color = Color {
  r: 255,
  g: 165,
  b: 0,
  a: 255,
};
pub const COLOR_PINK: Color = Color {
  r: 255,
  g: 192,
  b: 203,
  a: 255,
};
pub const COLOR_TRANSPARENT: Color = Color {
  r: 0,
  g: 0,
  b: 0,
  a: 0,
};

// NAPI exports
#[napi]
pub fn create_color(r: u8, g: u8, b: u8, a: u8) -> Color {
  Color::new(r, g, b, a)
}

#[napi]
pub fn color_red() -> Color {
  COLOR_RED
}
#[napi]
pub fn color_green() -> Color {
  COLOR_GREEN
}
#[napi]
pub fn color_blue() -> Color {
  COLOR_BLUE
}
#[napi]
pub fn color_black() -> Color {
  COLOR_BLACK
}
#[napi]
pub fn color_white() -> Color {
  COLOR_WHITE
}
#[napi]
pub fn color_yellow() -> Color {
  COLOR_YELLOW
}
#[napi]
pub fn color_cyan() -> Color {
  COLOR_CYAN
}
#[napi]
pub fn color_magenta() -> Color {
  COLOR_MAGENTA
}
#[napi]
pub fn color_gray() -> Color {
  COLOR_GRAY
}
#[napi]
pub fn color_dark_gray() -> Color {
  COLOR_DARK_GRAY
}
#[napi]
pub fn color_light_gray() -> Color {
  COLOR_LIGHT_GRAY
}
#[napi]
pub fn color_orange() -> Color {
  COLOR_ORANGE
}
#[napi]
pub fn color_pink() -> Color {
  COLOR_PINK
}
#[napi]
pub fn color_transparent() -> Color {
  COLOR_TRANSPARENT
}

#[napi]
pub fn color_to_rgba(color: Color) -> Vec<u8> {
  vec![color.r, color.g, color.b, color.a]
}

#[napi]
pub fn color_to_hex(color: Color) -> String {
  color.to_hex()
}

#[napi]
pub fn color_to_rgb_hex(color: Color) -> String {
  color.to_rgb_hex()
}

#[napi]
pub fn blend_colors(foreground: Color, background: Color) -> Color {
  foreground.blend(&background)
}

#[napi]
pub fn lerp_colors(color1: Color, color2: Color, t: f64) -> Color {
  color1.lerp(&color2, t)
}
