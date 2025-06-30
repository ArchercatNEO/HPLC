use plotters::{
    element::{Drawable, PointCollection},
    prelude::*,
};

use crate::vector::*;

#[derive(Clone, Debug, Default)]
pub enum PeakType {
    #[default]
    Standard,
    Shoulder(RGBColor),
    Reference,
}

#[derive(Clone, Debug, Default)]
pub struct Peak {
    pub start: Point2D,
    pub retention_point: Point2D,
    pub end: Point2D,
    pub height: f32,
    pub area: f32,
    pub concentration: f32,
    pub lipid: Option<String>,
    pub peak_type: PeakType,
}

impl<'a> PointCollection<'a, Point2D> for &'a Peak {
    type Point = &'a Point2D;
    type IntoIter = [&'a Point2D; 2];

    fn point_iter(self) -> Self::IntoIter {
        [&self.start, &self.retention_point]
    }
}

impl<DB: DrawingBackend> Drawable<DB> for Peak {
    fn draw<I: Iterator<Item = (i32, i32)>>(
        &self,
        mut pos: I,
        backend: &mut DB,
        parent_dim: (u32, u32),
    ) -> Result<
        (),
        plotters_iced::plotters_backend::DrawingErrorKind<<DB as DrawingBackend>::ErrorType>,
    > {
        match self.peak_type {
            PeakType::Standard => {
                backend.draw_circle(pos.next().unwrap(), 3, &BLUE, true)?;

                let retention = pos.next().unwrap();
                let text = self.lipid.as_ref().map_or("Unknown", |label| &label);

                backend.draw_text(
                    &text,
                    &("sans-serif", 10).into_text_style(&parent_dim),
                    retention,
                )?;
                backend.draw_circle(retention, 3, &GREEN, true)?;
            }
            PeakType::Shoulder(color) => {
                let start = pos.next().unwrap();
                let retention = pos.next().unwrap();

                backend.draw_circle(start, 3, &color, true)?;
                backend.draw_circle(retention, 3, &color, true)?;

                let text = self.lipid.as_ref().map_or("Unknown", |label| &label);
                backend.draw_text(
                    &text,
                    &("sans-serif", 10).into_text_style(&parent_dim),
                    retention,
                )?;
            }
            PeakType::Reference => {
                let retention = pos.next().unwrap();
                let text = self.lipid.as_ref().map_or("Unknown", |label| &label);

                backend.draw_text(
                    &text,
                    &("sans-serif", 10).into_text_style(&parent_dim),
                    retention,
                )?;
                backend.draw_circle(retention, 3, &RED, true)?;
            }
        }

        Ok(())
    }
}
