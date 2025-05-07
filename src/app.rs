use std::ops::Range;

use iced::{alignment::Horizontal, widget::{button, column, row, slider, text}, Element, Length, Task};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

use crate::{chromatography::Chromatography, vector::*};

#[derive(Debug)]
pub struct App {
    chromatography: Chromatography,
    chart_start: f32,
    chart_end: f32
}

impl Default for App {
    fn default() -> Self {
        let mut app = App {
            chromatography: Chromatography::default(),
            chart_start: 9.0,
            chart_end: 45.0
        };

        app.chromatography.set_data_range(9.0..45.0);
        app
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Done,
    DataFile,
    DataLoad(Vec<Point2D>),
    ReferenceFile,
    ReferenceLoad(Vec<(f32, String)>),
    ChartRange(Range<f32>),
    NoiseReduction(f32)
}

impl App {
    pub fn view(&self) -> Element<Message> {
        let load_data_file = column![
            button("Load Raw Data File").on_press(Message::DataFile)
        ];

        let load_reference_file = column![
            button("Load Lipid Reference File").on_press(Message::ReferenceFile)
        ];

        let chart_range = {
            let start = slider(0.0..=60.0, self.chart_start, |start| {
                let clamped = start.min(self.chart_end);
                Message::ChartRange(clamped..self.chart_end)
            });

            let end = slider(0.0..=60.0, self.chart_end, |end| {
                let clamped = end.max(self.chart_start);
                Message::ChartRange(self.chart_start..clamped)
            });
            
            column![
                text("Chart Start"),
                start,
                text("Chart End"),
                end
            ].max_width(200)
        };

        let noise_tolerance = {
            let display = format!("Noise Tolerance: {}", self.chromatography.noise_reduction);
            let label = text(display).align_x(Horizontal::Center);

            let noise_slider = slider(
                0.0..=10.0,
                self.chromatography.noise_reduction,
                Message::NoiseReduction
            );
        
            column![label, noise_slider].max_width(200)
        };

        let options = column![
            load_data_file,
            load_reference_file,
            chart_range,
            noise_tolerance
        ].max_width(200);

        let chart = ChartWidget::new(self)
            .height(Length::Fill)
            .width(Length::Fill);

        row![options, chart].into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Done => {
                Task::none()
            }
            Message::DataFile => {
                let task = rfd::AsyncFileDialog::new()
                    .set_directory("~/src/HPLC")
                    .add_filter("text", &["tsv"])
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
                    .set_directory("~/src/HPLC")
                    .add_filter("text", &["tsv"])
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
        }
    }
}

impl Chart<Message> for App {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {        
        let range = self.chromatography.get_data_range();

        let mut chart = builder
            .caption("HPLC", ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(range, -2f32..150f32)
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
            .draw_series(LineSeries::new(self.chromatography.baseline.clone(), &GREEN))
            .expect("failed to draw series")
            .label("baseline")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));

        let elements= self.chromatography.peaks.clone();

        chart
            .draw_series(elements)
            .expect("failed to draw series")
            .label("data")
            .legend(|(x, y)| Circle::new((x, y), 5, &BLUE));
    }
}
