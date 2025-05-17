use std::fs;
use std::ops::Range;
use std::path::PathBuf;

use iced::alignment::Vertical;
use iced::mouse::{self, Cursor, Event as MouseEvent};
use iced::widget::canvas::Event as CanvasEvent;
use iced::widget::{scrollable, toggler};
use iced::{
    Element, Length, Task,
    alignment::Horizontal,
    widget::{button, column, row, slider, text},
};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

use crate::{chromatography::Chromatography, vector::*};

#[derive(Debug)]
pub struct App {
    chromatography: Chromatography,
    chart_start: f32,
    chart_end: f32,
    show_unknowns: bool,
    zoom_x: f32,
    zoom_y: f32
}

impl Default for App {
    fn default() -> Self {
        let mut app = App {
            chromatography: Chromatography::default(),
            chart_start: 9.0,
            chart_end: 34.5,
            show_unknowns: false,
            zoom_x: 0.0,
            zoom_y: 0.0
        };

        app.chromatography
            .set_data_range(app.chart_start..app.chart_end);
        app
    }
}

pub struct AppState {
    pub mouse_inside: bool,
    pub mouse_pressed: bool,
    pub horizontal: bool,
    pub vertical: bool,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_zoom: f32,
    pub y_zoom: f32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mouse_inside: false,
            mouse_pressed: false,
            horizontal: true,
            vertical: true,
            mouse_x: 0.0,
            mouse_y: 0.0,
            x_offset: 0.0,
            y_offset: 0.0,
            x_zoom: 1.0,
            y_zoom: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Done,
    DataFile,
    DataLoad(Vec<Point2D>),
    ReferenceFile,
    ReferenceLoad(Vec<(f32, String)>),
    ExportFile,
    ExportFileContent(PathBuf),
    ChartRange(Range<f32>),
    NoiseReduction(f32),
    ShowUnknowns(bool),
    ZoomX(f32),
    ZoomY(f32),
}

impl App {
    pub fn view(&self) -> Element<Message> {
        let load_data_file = button("Load Raw Data File").on_press(Message::DataFile);

        let load_reference_file =
            button("Load Lipid Reference File").on_press(Message::ReferenceFile);

        let export_file = button("Export Table").on_press(Message::ExportFile);

        let chart_range = {
            let start = slider(0.0..=60.0, self.chart_start, |start| {
                let clamped = start.min(self.chart_end);
                Message::ChartRange(clamped..self.chart_end)
            });

            let end = slider(0.0..=60.0, self.chart_end, |end| {
                let clamped = end.max(self.chart_start);
                Message::ChartRange(self.chart_start..clamped)
            });

            let label_start = format!("Chart Start: {}", self.chart_start);
            let label_end = format!("Chart End: {}", self.chart_end);
            column![text(label_start), start, text(label_end), end]
        };

        let noise_tolerance = {
            let display = format!("Noise Tolerance: {}", self.chromatography.noise_reduction);
            let label = text(display).align_x(Horizontal::Center);

            let noise_slider = slider(
                0.0..=10.0,
                self.chromatography.noise_reduction,
                Message::NoiseReduction,
            )
            .step(0.1);

            column![label, noise_slider]
        };

        let unknown_lipid = {
            let label = text("Show Unknown Lipids");
            let slide = toggler(self.show_unknowns).on_toggle(Message::ShowUnknowns);
            row![label, slide]
        };

        let options = column![
            load_data_file,
            load_reference_file,
            export_file,
            chart_range,
            noise_tolerance,
            unknown_lipid
        ]
        .width(250);

        let zoom_x = slider(0.0..=100.0, self.zoom_x, Message::ZoomX);
        let zoom_x_content = format!("X Zoom: {}%", f32::powf(1.1, self.zoom_x) * 100.0);
        let zoom_x_label = text(zoom_x_content);
        let zoom_y = slider(0.0..=100.0, self.zoom_y, Message::ZoomY);
        let zoom_y_content = format!("Y Zoom: {}%", f32::powf(1.1, self.zoom_y) * 100.0);
        let zoom_y_label = text(zoom_y_content);

        let options2 = column![
            zoom_x_label,
            zoom_x,
            zoom_y_label,
            zoom_y
        ].width(250);

        let table = {
            let header = row![
                text("Time (s)").center().width(80),
                text("Lipid").center().width(200),
                text("Area (m^2)").center().width(150)
            ]
            .spacing(20);
            let mut inner = column![header];

            for peak in &self.chromatography.peaks {
                let time = crate::round_to_precision(peak.turning_point.x(), 2);
                let time_label = text(time).center().width(80);
                let area = crate::round_to_precision(peak.area, 2);
                let area_label = text(area).center().width(150);

                let label = if let Some(lipid) = &peak.lipid {
                    text(lipid).center().width(200)
                } else {
                    text("Unknown").center().width(200)
                };

                if self.show_unknowns || peak.lipid != None {
                    let row = row![time_label, label, area_label]
                        .spacing(20)
                        .align_y(Vertical::Center);
                    inner = inner.push(row);
                }
            }
            scrollable(inner)
        };

        let footer = row![options, options2, table].height(250);

        let chart = ChartWidget::new(self)
            .height(Length::Fill)
            .width(Length::Fill);

        column![chart, footer].into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Done => Task::none(),
            Message::DataFile => {
                let task = rfd::AsyncFileDialog::new()
                    .add_filter("any", &["*"])
                    .add_filter("text", &["arw", "csv", "tsv", "txt"])
                    .pick_file();

                Task::perform(task, |maybe_handle| {
                    if let Some(handle) = maybe_handle {
                        //TODO: display loaded file path
                        let path = handle.path();
                        let data = crate::parse_file(path, crate::parse_line_as_data);

                        Message::DataLoad(data)
                    } else {
                        Message::Done
                    }
                })
            }
            Message::DataLoad(data) => {
                self.chromatography.set_data(data);

                Task::none()
            }
            Message::ReferenceFile => {
                let task = rfd::AsyncFileDialog::new()
                    .add_filter("any", &["*"])
                    .add_filter("text", &["arw", "csv", "tsv", "txt"])
                    .pick_file();

                Task::perform(task, |maybe_handle| {
                    if let Some(handle) = maybe_handle {
                        //TODO: display loaded file path
                        let path = handle.path();
                        let data = crate::parse_file(path, crate::parse_line_as_lipids);

                        Message::ReferenceLoad(data)
                    } else {
                        Message::Done
                    }
                })
            }
            Message::ReferenceLoad(data) => {
                self.chromatography.set_lipid_master_table(data);

                Task::none()
            }
            Message::ExportFile => {
                let task = rfd::AsyncFileDialog::new()
                    .set_file_name("table.arw")
                    .save_file();

                Task::perform(task, |maybe_handle| {
                    if let Some(handle) = maybe_handle {
                        let path = handle.path().to_owned();
                        Message::ExportFileContent(path)
                    } else {
                        Message::Done
                    }
                })
            }
            Message::ExportFileContent(path) => {
                let content = self
                    .chromatography
                    .peaks
                    .iter()
                    .map(|peak| {
                        let label = peak.lipid.clone().unwrap_or("Unknown".to_string());
                        format!("{}\t{}\t{}\n", label, peak.area, peak.turning_point.x())
                    })
                    .fold(
                        "lipid\tarea\tturning point\n".to_string(),
                        |accum: String, peak| accum + &peak,
                    );
                fs::write(path, content).expect("Cannot write there");

                Task::none()
            }
            Message::ChartRange(range) => {
                self.chart_start = range.start;
                self.chart_end = range.end;
                self.chromatography.set_data_range(range);

                Task::none()
            }
            Message::NoiseReduction(value) => {
                self.chromatography.set_noise_reduction(value);

                Task::none()
            }
            Message::ShowUnknowns(show) => {
                self.show_unknowns = show;

                Task::none()
            }
            Message::ZoomX(zoom) => { 
                self.zoom_x = zoom;
                
                Task::none()
            }
            Message::ZoomY(zoom) => { 
                self.zoom_y = zoom;

                Task::none()
            }
        }
    }
}

impl Chart<Message> for App {
    type State = AppState;

    fn build_chart<DB: DrawingBackend>(&self, state: &Self::State, mut builder: ChartBuilder<DB>) {
        let range = self.chromatography.get_data_range();
        let scaled_x_range = {
            let start = range.start + state.x_offset;
            let end = range.end + state.x_offset;
            let middle = (start + end) / 2.0;

            let power = f32::powf(1.1, -self.zoom_x);
            let scaled_start = (start - middle) * state.x_zoom * power + middle;
            let scaled_end = (end - middle) * state.x_zoom * power + middle;
            scaled_start..scaled_end
        };

        let scaled_y_range = {
            let min = state.y_offset;
            let max = self.chromatography.get_highest_point() + state.y_offset;
            let middle = (min + max) / 2.0;

            let power = f32::powf(1.1, -self.zoom_y);
            let scaled_min = (min - middle) * state.y_zoom * power + middle;
            let scaled_max = (max - middle) * state.y_zoom * power + middle;

            scaled_min..scaled_max
        };

        let mut chart = builder
            .caption("HPLC", ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(scaled_x_range, scaled_y_range)
            .expect("failed to build chart");

        chart
            .configure_mesh()
            .draw()
            .expect("failed to configure chart");

        let data_series = LineSeries::new(self.chromatography.get_data(), &RED);
        chart
            .draw_series(data_series)
            .expect("failed to draw series")
            .label("data")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .draw_series(LineSeries::new(
                self.chromatography.baseline.clone(),
                &GREEN,
            ))
            .expect("failed to draw series")
            .label("baseline")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));

        let elements = self.chromatography.peaks.clone();

        chart
            .draw_series(elements)
            .expect("failed to draw series")
            .label("data")
            .legend(|(x, y)| Circle::new((x, y), 5, &BLUE));
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: CanvasEvent,
        _bounds: iced::Rectangle,
        _cursor: Cursor,
    ) -> (iced::event::Status, Option<Message>) {
        use iced::event::Status;
        use iced::keyboard::Event as KeyboardEvent;

        if let CanvasEvent::Mouse(ev) = event {
            let result = match ev {
                MouseEvent::ButtonPressed(btn) => match btn {
                    mouse::Button::Left => {
                        state.mouse_pressed = state.mouse_inside;
                        (Status::Captured, None)
                    }
                    _ => (Status::Ignored, None),
                },
                MouseEvent::ButtonReleased(btn) => match btn {
                    mouse::Button::Left => {
                        state.mouse_pressed = false;
                        (Status::Captured, None)
                    }
                    _ => (Status::Ignored, None),
                },
                MouseEvent::CursorMoved { position } => {
                    if state.mouse_pressed && state.mouse_inside {
                        if state.horizontal {
                            let delta_x = position.x - state.mouse_x;
                            state.x_offset -= delta_x * state.x_zoom * 0.05;
                        }

                        if state.vertical {
                            let delta_y = position.y - state.mouse_y;
                            state.y_offset += delta_y + state.y_zoom * 0.05;
                        }
                    }

                    state.mouse_x = position.x;
                    state.mouse_y = position.y;
                    state.mouse_inside = _bounds.contains(position);
                    (Status::Captured, None)
                }
                MouseEvent::WheelScrolled { delta } => {
                    if state.mouse_inside {
                        match delta {
                            mouse::ScrollDelta::Lines { x: _, y } => {
                                if state.horizontal {
                                    state.x_zoom *= 1.0 - y * 0.1;
                                }
    
                                if state.vertical {
                                    state.y_zoom *= 1.0 - y * 0.1;
                                }
                            }
                            mouse::ScrollDelta::Pixels { x: _, y: _ } => {}
                        }
                    }
                    (Status::Captured, None)
                }
                _ => (Status::Ignored, None)
            };

            return result;
        }

        if let CanvasEvent::Keyboard(keyboard) = event {
            let result = match keyboard {
                KeyboardEvent::ModifiersChanged(mods) => {
                    state.horizontal = !mods.control();
                    state.vertical = !mods.alt();
                    (Status::Captured, None)
                }
                _ => (Status::Ignored, None),
            };

            return result;
        }

        (Status::Ignored, None)
    }
}
