#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Quadratic {
    a: f32,
    b: f32,
    c: f32,
}

impl Quadratic {
    pub fn new(a: f32, b: f32, c: f32) -> Self {
        Self { a, b, c }
    }

    pub fn evaluate(&self, x: f32) -> f32 {
        self.a * f32::powi(x, 2) + self.b * x + self.c
    }
}
