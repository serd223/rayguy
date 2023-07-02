pub use softbuffer;

pub struct WrapBuffer<'a> {
    inner: softbuffer::Buffer<'a>,
    surface_size: (usize, usize),
    window_size: (usize, usize),
    ratio: (usize, usize),
}

impl<'a> WrapBuffer<'a> {
    pub fn new(
        buffer: softbuffer::Buffer<'a>,
        surface_size: (usize, usize),
        window_size: (usize, usize),
    ) -> Self {
        let ratio = (
            window_size.0 / surface_size.0,
            window_size.1 / surface_size.1,
        );
        Self {
            inner: buffer,
            surface_size,
            window_size,
            ratio,
        }
    }
    pub fn resize_surface(&mut self, new_size: (usize, usize)) {
        self.ratio.0 = new_size.0 / self.window_size.0;
        self.ratio.1 = new_size.1 / self.window_size.1;
        self.surface_size = new_size;
    }

    pub fn resize_window(&mut self, new_size: (usize, usize)) {
        self.ratio.0 = self.surface_size.0 / new_size.0;
        self.ratio.1 = self.surface_size.1 / new_size.1;
        self.window_size = new_size;
    }

    pub fn set(&mut self, x: usize, y: usize, color: Color) {
        self.set_raw(x, y, color.as_pixel());
    }
    pub fn set_raw(&mut self, x: usize, y: usize, val: u32) {
        if matches!(self.ratio, (0, 0)) {
            let index = self.window_size.0 * y + x;
            if index < self.inner.len() {
                self.inner[index] = val;
            }
            return;
        }
        let ratio = (
            if self.ratio.0 > 0 { self.ratio.0 } else { 1 },
            if self.ratio.1 > 0 { self.ratio.1 } else { 1 },
        );
        let start = x * ratio.0;
        let end = (x + 1) * ratio.0;
        for i in start..end {
            for j in y * ratio.1..(y + 1) * ratio.1 {
                let idx = i + j * self.window_size.0;
                if idx < self.inner.len() {
                    self.inner[i + j * self.window_size.0] = val;
                }
            }
        }
    }
    pub fn vert_line(&mut self, x: usize, draw_start: usize, draw_end: usize, color: Color) {
        let start = draw_start.min(draw_end);
        let end = draw_start.max(draw_end);
        let pixel = color.as_pixel();
        for y in start..end + 1 {
            self.set_raw(x, y, pixel);
        }
    }

    pub fn present(self) -> Result<(), softbuffer::SoftBufferError> {
        self.inner.present()
    }
}

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
