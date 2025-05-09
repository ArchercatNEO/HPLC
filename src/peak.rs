use plotters::{
    element::{Drawable, PointCollection},
    prelude::*,
};

use crate::vector::*;

#[derive(Clone, Debug, Default)]
pub struct Peak {
    pub start: Point2D,
    pub turning_point: Point2D,
    pub end: Point2D,
    pub area: f32,
    pub lipid: Option<String>,
}

impl Peak {
    // area of trapezium = 1/2 * (a + b) * h
    pub fn area(&mut self, gradient: f32, offset: f32) {
        let start_baseline = self.start.x() * gradient + offset;
        let midpoint_baseline = self.turning_point.x() * gradient + offset;
        let end_baseline = self.end.x() * gradient + offset;

        let left = {
            let height = self.turning_point.x() - self.start.x();
            let a = self.start.y() - start_baseline;
            let b = self.turning_point.y() - midpoint_baseline;
            0.5 * (a + b) * height
        };

        let right = {
            let height = self.end.x() - self.turning_point.x();
            let a = self.turning_point.y() - midpoint_baseline;
            let b = self.end.y() - end_baseline;
            0.5 * (a + b) * height
        };

        self.area = left + right;
        if self.area < 0.0 {
            println!(
                "start: {}, mid: {}, end: {}",
                start_baseline, midpoint_baseline, end_baseline
            );
        }
    }
}

impl<'a> PointCollection<'a, Point2D> for &'a Peak {
    type Point = &'a Point2D;
    type IntoIter = [&'a Point2D; 3];

    fn point_iter(self) -> Self::IntoIter {
        [&self.start, &self.turning_point, &self.end]
    }
}

impl<DB: DrawingBackend> Drawable<DB> for Peak {
    fn draw<I: Iterator<Item = (i32, i32)>>(
        &self,
        pos: I,
        backend: &mut DB,
        parent_dim: (u32, u32),
    ) -> Result<
        (),
        plotters_iced::plotters_backend::DrawingErrorKind<<DB as DrawingBackend>::ErrorType>,
    > {
        let mut which = 0;
        for point in pos {
            backend.draw_circle(point, 5, &BLUE, true)?;

            if which == 1 {
                let text = if let Some(label) = &self.lipid {
                    label
                } else {
                    "Unknown"
                };

                backend.draw_text(
                    text,
                    &("sans-serif", 10).into_text_style(&parent_dim),
                    point,
                )?;
            }

            which += 1;
        }

        Ok(())
    }
}
