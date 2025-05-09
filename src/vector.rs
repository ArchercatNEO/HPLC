pub type Point2D = (f32, f32);

pub trait Vector2 {
    fn new(x: f32, y: f32) -> Self;

    fn x(self: &Self) -> f32;
    fn y(self: &Self) -> f32;

    fn gradient(self: &Self, rhs: &Self) -> f32 {
        let delta_x = rhs.x() - self.x();
        let delta_y = rhs.y() - self.y();
        delta_y / delta_x
    }
}

impl Vector2 for Point2D {
    fn new(x: f32, y: f32) -> Self {
        (x, y)
    }

    fn x(&self) -> f32 {
        self.0
    }
    fn y(&self) -> f32 {
        self.1
    }
}
