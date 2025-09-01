pub type Point2D = (f64, f64);

pub trait Vector2 {
    fn new(x: f64, y: f64) -> Self;

    fn x(self: &Self) -> f64;
    fn y(self: &Self) -> f64;

    fn gradient(self: &Self, rhs: &Self) -> f64 {
        let delta_x = rhs.x() - self.x();
        let delta_y = rhs.y() - self.y();
        delta_y / delta_x
    }
}

impl Vector2 for Point2D {
    fn new(x: f64, y: f64) -> Self {
        (x, y)
    }

    fn x(&self) -> f64 {
        self.0
    }
    fn y(&self) -> f64 {
        self.1
    }
}
