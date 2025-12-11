use iced::event::Status;
use iced::widget::canvas;
use iced::{Point, keyboard, mouse};
use plotters::element::{Drawable, PointCollection};
use plotters::prelude::*;
use plotters_iced::Chart;

use crate::chromatography::{Chromatography, ComponentFilter};
use crate::component::Component;
use crate::vector::Point2D;

#[derive(Debug, Clone)]
pub struct ChromatogramState {
    mouse_inside: bool,
    mouse_pressed: bool,
    mouse_position: Point<f64>,
    ctrl_pressed: bool,
    alt_pressed: bool,
    local_zoom: Point<f64>,
    local_offset: Point<f64>,
}

impl Default for ChromatogramState {
    fn default() -> Self {
        Self {
            mouse_inside: false,
            mouse_pressed: false,
            mouse_position: Point::new(0.0, 0.0),
            ctrl_pressed: false,
            alt_pressed: false,
            local_zoom: Point::new(1.0, 1.0),
            local_offset: Point::new(0.0, 0.0),
        }
    }
}

impl<'a> PointCollection<'a, Point2D> for &'a Component {
    type Point = &'a Point2D;
    type IntoIter = [&'a Point2D; 2];

    fn point_iter(self) -> Self::IntoIter {
        match self {
            Component::Unknown(peak) => [&peak.start, &peak.retention_point],
            Component::Located(peak, _) => [&peak.start, &peak.retention_point],
            Component::Reference(_) => todo!(),
        }
    }
}

impl<DB: DrawingBackend> Drawable<DB> for Component {
    fn draw<I: Iterator<Item = (i32, i32)>>(
        &self,
        mut pos: I,
        backend: &mut DB,
        parent_dim: (u32, u32),
    ) -> Result<
        (),
        plotters_iced::plotters_backend::DrawingErrorKind<<DB as DrawingBackend>::ErrorType>,
    > {
        backend.draw_circle(pos.next().unwrap(), 3, &BLUE, true)?;

        let retention = pos.next().unwrap();
        backend.draw_circle(retention, 3, &GREEN, true)?;

        let text = self.point_label();
        backend.draw_text(
            &text,
            &("sans-serif", 10).into_text_style(&parent_dim),
            retention,
        )?;

        Ok(())
    }
}

impl Chart<()> for Chromatography {
    type State = ChromatogramState;

    fn build_chart<DB: plotters::prelude::DrawingBackend>(
        &self,
        state: &Self::State,
        mut builder: plotters::prelude::ChartBuilder<DB>,
    ) {
        let range = self.get_data_range();
        let scaled_range_x = {
            let start = range.start + state.local_offset.x;
            let end = range.end + state.local_offset.x;
            let middle = (start + end) / 2.0;

            let combined_zoom = state.local_zoom.x * (1.0 / self.global_zoom.x);
            let scaled_start = (start - middle) * combined_zoom + middle;
            let scaled_end = (end - middle) * combined_zoom + middle;
            scaled_start..scaled_end
        };

        let scaled_range_y = {
            let min = state.local_offset.y;
            let max = self.get_highest_point() + state.local_offset.y;
            let middle = self.get_highest_point() / 2.0 + state.local_offset.y;

            let combined_zoom = state.local_zoom.y * (1.0 / self.global_zoom.y);
            let scaled_start = (min - middle) * combined_zoom + middle;
            let scaled_end = (max - middle) * combined_zoom + middle;
            scaled_start..scaled_end
        };

        let mut chart = builder
            .caption(&self.title, ("sans-serif", 30).into_font())
            .margin(40)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(scaled_range_x, scaled_range_y)
            .expect("failed to build chart");

        chart
            .configure_mesh()
            .draw()
            .expect("failed to configure chart");

        let data_series = LineSeries::new(self.get_data(), &RED);

        chart
            .draw_series(data_series)
            .expect("failed to draw series")
            .label("data")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        let baseline_series = LineSeries::new(self.baseline.clone(), &GREEN);
        chart
            .draw_series(baseline_series)
            .expect("failed to draw series")
            .label("baseline")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));

        chart
            .draw_series(self.get_components(&ComponentFilter::EXISTING_ONLY).clone())
            .expect("failed to draw series")
            .label("peaks")
            .legend(|(x, y)| Circle::new((x, y), 5, &BLUE));
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> (Status, Option<()>) {
        if let canvas::Event::Mouse(mouse_ev) = event {
            match mouse_ev {
                mouse::Event::ButtonPressed(btn) => {
                    if state.mouse_inside && btn == mouse::Button::Left {
                        state.mouse_pressed = true;
                    }
                }
                mouse::Event::ButtonReleased(btn) => {
                    if btn == mouse::Button::Left {
                        state.mouse_pressed = false;
                    }
                }
                mouse::Event::CursorMoved { position } => {
                    let positionf64: Point<f64> = Point {
                        x: position.x as f64,
                        y: position.y as f64,
                    };

                    if state.mouse_inside && state.mouse_pressed {
                        let difference = positionf64 - state.mouse_position;

                        if !state.ctrl_pressed {
                            state.local_offset.x -= difference.x * 0.05;
                        }
                        if !state.alt_pressed {
                            state.local_offset.y += difference.y * 0.5;
                        }
                    }

                    state.mouse_inside = bounds.contains(position);
                    state.mouse_position = positionf64;
                }
                mouse::Event::WheelScrolled { delta } => {
                    if state.mouse_inside {
                        if let mouse::ScrollDelta::Lines { x: _, y } = delta {
                            if !state.ctrl_pressed {
                                state.local_zoom.x *= 1.0 - (y as f64) * 0.1;
                            }
                            if !state.alt_pressed {
                                state.local_zoom.y *= 1.0 - (y as f64) * 0.1;
                            }
                        }
                    }
                }
                _ => {
                    return (Status::Ignored, None);
                }
            }

            return (Status::Captured, None);
        }

        if let canvas::Event::Keyboard(keyboard_ev) = event {
            match keyboard_ev {
                keyboard::Event::ModifiersChanged(mods) => {
                    state.ctrl_pressed = mods.control();
                    state.alt_pressed = mods.alt();
                    return (Status::Captured, None);
                }
                _ => {}
            }
        }

        (Status::Ignored, None)
    }
}
