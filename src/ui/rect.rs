#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    pub fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
    pub fn x(&self, x: u16) -> Self {
        Self::new(x, self.y, self.width, self.height)
    }
    pub fn y(&self, y: u16) -> Self {
        Self::new(self.x, y, self.width, self.height)
    }
    pub fn w(&self, w: u16) -> Self {
        Self::new(self.x, self.y, w, self.height)
    }
    pub fn h(&self, h: u16) -> Self {
        Self::new(self.x, self.y, self.width, h)
    }
    pub fn intersects(&self, rect: &Rect) -> bool {
        self.x < rect.x + rect.width
            && rect.x < self.x + self.width
            && self.y < rect.y + rect.height
            && rect.y < self.y + self.height
    }
}
