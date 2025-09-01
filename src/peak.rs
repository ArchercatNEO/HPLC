use plotters::{
    element::{Drawable, PointCollection},
    prelude::*,
};

use crate::{reference::Reference, vector::*};

#[derive(Clone, Debug, Default, PartialEq)]
pub enum PeakType {
    #[default]
    Unknown,
    Common(Reference),
    Missing(Reference),
}

#[derive(Clone, Debug, Default)]
pub struct Peak {
    pub start: Point2D,
    pub retention_point: Point2D,
    pub gu: Option<f64>,
    pub end: Point2D,
    pub height: f64,
    pub area: f64,
    pub peak_type: PeakType,
}

impl Peak {
    fn format_label(&self) -> String {
        let name = match &self.peak_type {
            PeakType::Unknown => "[Unknown]",
            PeakType::Common(reference) => reference
                .name
                .as_ref()
                .map_or("[Unnamed]", |string| &string),
            PeakType::Missing(reference) => reference
                .name
                .as_ref()
                .map_or("[Unnamed]", |string| &string),
        };

        match &self.gu {
            Some(gu) => format!("[{}, {:.3}, {:.3}]", name, self.retention_point.x(), gu),
            None => format!("[{}, {:.3}]", name, self.retention_point.x()),
        }
    }
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
        match &self.peak_type {
            PeakType::Unknown => {
                // Peak which has no reference
                backend.draw_circle(pos.next().unwrap(), 3, &BLUE, true)?;

                let retention = pos.next().unwrap();
                backend.draw_circle(retention, 3, &GREEN, true)?;

                let text = self.format_label();
                backend.draw_text(
                    &text,
                    &("sans-serif", 10).into_text_style(&parent_dim),
                    retention,
                )?;
            }
            PeakType::Common(_) => {
                // Peak which is both in data and reference

                backend.draw_circle(pos.next().unwrap(), 3, &BLUE, true)?;

                let retention = pos.next().unwrap();
                backend.draw_circle(retention, 3, &GREEN, true)?;

                let text = self.format_label();
                backend.draw_text(
                    &text,
                    &("sans-serif", 10).into_text_style(&parent_dim),
                    retention,
                )?;
            }
            PeakType::Missing(_) => {
                // Peak that is missing from the data but in the reference
            }
        }

        Ok(())
    }
}
