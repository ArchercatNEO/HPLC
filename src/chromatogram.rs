use iced::event::Status;
use iced::widget::canvas;
use iced::{Point, keyboard, mouse};
use plotters::prelude::*;
use plotters_iced::Chart;

use crate::chromatography::Chromatography;

#[derive(Debug, Clone)]
pub struct ChromatogramState {
    mouse_inside: bool,
    mouse_pressed: bool,
    mouse_position: Point,
    ctrl_pressed: bool,
    alt_pressed: bool,
    local_zoom: Point,
    local_offset: Point,
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

            let power = f32::powf(0.9, self.global_zoom.x);
            let scaled_start = (start - middle) * state.local_zoom.x * power + middle;
            let scaled_end = (end - middle) * state.local_zoom.x * power + middle;
            scaled_start..scaled_end
        };

        let scaled_range_y = {
            let min = state.local_offset.y;
            let max = self.get_highest_point() + state.local_offset.y;
            let middle = self.get_highest_point() / 2.0 + state.local_offset.y;

            let power = f32::powf(0.9, self.global_zoom.y);
            let scaled_start = (min - middle) * state.local_zoom.y * power + middle;
            let scaled_end = (max - middle) * state.local_zoom.y * power + middle;
            scaled_start..scaled_end
        };

        let title = self.title.as_ref().map_or("HPLC", |string| &string);

        let mut chart = builder
            .caption(title, ("sans-serif", 30).into_font())
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
            .draw_series(self.peaks.clone())
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
                    if state.mouse_inside && state.mouse_pressed {
                        let difference = position - state.mouse_position;

                        if !state.ctrl_pressed {
                            state.local_offset.x -= difference.x * state.local_zoom.x * 0.05;
                        }
                        if !state.alt_pressed {
                            state.local_offset.y += difference.y * state.local_zoom.y * 0.5;
                        }
                    }

                    state.mouse_inside = bounds.contains(position);
                    state.mouse_position = position;
                }
                mouse::Event::WheelScrolled { delta } => {
                    if state.mouse_inside {
                        if let mouse::ScrollDelta::Lines { x: _, y } = delta {
                            if !state.ctrl_pressed {
                                state.local_zoom.x *= 1.0 - y * 0.1;
                            }
                            if !state.alt_pressed {
                                state.local_zoom.y *= 1.0 - y * 0.1;
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
