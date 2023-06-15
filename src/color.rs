#[derive(Clone)]
pub enum Color {
    Rgb(u8, u8, u8),
}

pub const RED: Color = Color::Rgb(255, 0, 0);
pub const GREEN: Color = Color::Rgb(0, 255, 0);
pub const BLUE: Color = Color::Rgb(0, 0, 255);
pub const WHITE: Color = Color::Rgb(255, 255, 255);
pub const YELLOW: Color = Color::Rgb(255, 255, 0);

impl Color {
    pub fn as_pixel(&self) -> u32 {
        match self {
            Self::Rgb(red, green, blue) => {
                ((*red as u32) << 16) | ((*green as u32) << 8) | (*blue as u32)
            }
        }
    }
}
