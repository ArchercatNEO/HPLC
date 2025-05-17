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
