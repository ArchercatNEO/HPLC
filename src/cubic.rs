use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq)]
pub struct Cubic {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

impl Cubic {
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self { a, b, c, d }
    }

    pub fn evaluate(&self, x: f32) -> f32 {
        self.a * x.powi(3) + self.b * x.powi(2) + self.c * x + self.d
    }
}

impl Debug for Cubic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formula = format!("{}x^3 + {}x^2 + {}x + {}", self.a, self.b, self.c, self.d);
        f.write_str(&formula)
    }
}

impl Default for Cubic {
    fn default() -> Self {
        Self {
            a: 0.0,
            b: 0.0,
            c: 1.0,
            d: 0.0,
        }
    }
}
