use std::fs;
use std::ops::Range;
use std::path::PathBuf;

use iced::{
    Element, Length, Point, Task,
    alignment::Horizontal,
    widget::{button, column, row, scrollable, slider, text, toggler},
};
use plotters_iced::ChartWidget;

use crate::{chromatography::Chromatography, vector::*};

#[derive(Debug)]
pub struct App {
    lipid_reference: Vec<(f32, String)>,
    samples: Vec<Chromatography>,
    sample_handle: Option<usize>,
    chart_start: f32,
    chart_end: f32,
    noise_reduction: f32,
    horizontal_deviation: f32,
    include_unknowns: bool,
    global_zoom: Point,
}

impl Default for App {
    fn default() -> Self {
        Self {
            lipid_reference: vec![],
            samples: vec![],
            sample_handle: None,
            chart_start: 9.0,
            chart_end: 34.5,
            noise_reduction: 0.3,
            horizontal_deviation: 0.5,
            include_unknowns: false,
            global_zoom: Point::new(0.0, 0.0),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Done,
    DataFile,
    DataLoad(Vec<PathBuf>),
    ReferenceFile,
    ReferenceLoad(Vec<(f32, String)>),
    ExportFile,
    ExportFileContent(PathBuf),
    ChartRange(Range<f32>),
    NoiseReduction(f32),
    HorizontalDeviation(f32),
    ShowUnknowns(bool),
    ZoomX(f32),
    ZoomY(f32),
    TabSwitch(usize),
}

impl From<()> for Message {
    fn from(_value: ()) -> Self {
        Message::Done
    }
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
            let display = format!("Noise Reduction: {}", self.noise_reduction);
            let label = text(display).align_x(Horizontal::Center);

            let noise_slider =
                slider(0.3..=0.6, self.noise_reduction, Message::NoiseReduction).step(0.01);

            column![label, noise_slider]
        };

        let horizontal_deviation = {
            let display = format!("Horizontal Deviation: {}", self.horizontal_deviation);
            let label = text(display).align_x(Horizontal::Center);

            let noise_slider = slider(
                0.0..=10.0,
                self.horizontal_deviation,
                Message::HorizontalDeviation,
            )
            .step(0.1);

            column![label, noise_slider]
        };

        let unknown_lipid = {
            let label = text("Show Unknown Lipids").align_x(Horizontal::Center);
            let slide = toggler(self.include_unknowns).on_toggle(Message::ShowUnknowns);
            column![label, slide]
        };

        let options = column![
            load_data_file,
            load_reference_file,
            export_file,
            chart_range,
            noise_tolerance,
            horizontal_deviation,
            unknown_lipid
        ]
        .width(250);

        let zoom_x = slider(0.0..=100.0, self.global_zoom.x, Message::ZoomX);
        let zoom_x_content = format!("X Zoom: {}%", f32::powf(1.1, self.global_zoom.x) * 100.0);
        let zoom_x_label = text(zoom_x_content);
        let zoom_y = slider(0.0..=100.0, self.global_zoom.y, Message::ZoomY);
        let zoom_y_content = format!("Y Zoom: {}%", f32::powf(1.1, self.global_zoom.y) * 100.0);
        let zoom_y_label = text(zoom_y_content);

        let options2 = column![zoom_x_label, zoom_x, zoom_y_label, zoom_y].width(250);

        let ui = if let Some(handle) = self.sample_handle {
            let tabs = {
                let mut buttons = column![];
                for i in 0..self.samples.len() {
                    let content = format!("{}", i);
                    let label = text(content);
                    let button = button(label).on_press(Message::TabSwitch(i));
                    buttons = buttons.push(button);
                }
                scrollable(buttons)
            };

            let sample = &self.samples[handle];
            let table = sample.into_table_element().map(Message::from);

            let footer = row![options, options2, table].height(250);
            let chart: Element<()> = ChartWidget::new(sample.clone())
                .height(Length::Fill)
                .width(Length::Fill)
                .into();

            let body = row![tabs, chart.map(Message::from)];
            column![body, footer]
        } else {
            let footer = row![options, options2].height(250);
            column![footer]
        };

        ui.into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Done => Task::none(),
            Message::DataFile => {
                let task = rfd::AsyncFileDialog::new()
                    .add_filter("any", &["*"])
                    .add_filter("text", &["arw", "csv", "tsv", "txt"])
                    .pick_files();

                Task::perform(task, |maybe_handle| {
                    if let Some(handles) = maybe_handle {
                        let paths = handles
                            .iter()
                            .map(|handle| handle.path().to_path_buf())
                            .collect();
                        Message::DataLoad(paths)
                    } else {
                        Message::Done
                    }
                })
            }
            Message::DataLoad(data) => {
                for path in data {
                    let content = crate::parse_file(path, crate::parse_line_as_data);
                    let mut sample = Chromatography::default();
                    sample.set_data(content);
                    sample.set_data_range(self.chart_start..self.chart_end);
                    sample.set_lipid_master_table(self.lipid_reference.clone());
                    sample.set_include_unknowns(self.include_unknowns);
                    sample.set_noise_reduction(self.noise_reduction);
                    sample.set_horizontal_deviation(self.horizontal_deviation);
                    self.samples.push(sample);
                }

                self.sample_handle = Some(self.samples.len() - 1);
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
                for sample in self.samples.iter_mut() {
                    sample.set_lipid_master_table(data.clone());
                }

                self.lipid_reference = data;
                Task::none()
            }
            Message::ExportFile => {
                let task = rfd::AsyncFileDialog::new()
                    .set_file_name("table.csv")
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
                let content = self.as_retention_table();
                fs::write(path, content).expect("Cannot write there");

                /* let mut content = String::new();
                for sample in &self.samples {
                    content += &sample.into_table();
                } */

                Task::none()
            }
            Message::ChartRange(range) => {
                self.chart_start = range.start;
                self.chart_end = range.end;
                for sample in self.samples.iter_mut() {
                    sample.set_data_range(range.clone());
                }

                Task::none()
            }
            Message::NoiseReduction(value) => {
                self.noise_reduction = value;
                for sample in self.samples.iter_mut() {
                    sample.set_noise_reduction(value);
                }

                Task::none()
            }
            Message::HorizontalDeviation(value) => {
                self.horizontal_deviation = value;
                for sample in self.samples.iter_mut() {
                    sample.set_horizontal_deviation(value);
                }

                Task::none()
            }
            Message::ShowUnknowns(show) => {
                self.include_unknowns = show;
                for sample in self.samples.iter_mut() {
                    sample.set_include_unknowns(show);
                }

                Task::none()
            }
            Message::ZoomX(zoom) => {
                self.global_zoom.x = zoom;
                for sample in self.samples.iter_mut() {
                    sample.global_zoom.x = zoom;
                }

                Task::none()
            }
            Message::ZoomY(zoom) => {
                self.global_zoom.y = zoom;
                for sample in self.samples.iter_mut() {
                    sample.global_zoom.y = zoom;
                }

                Task::none()
            }
            Message::TabSwitch(tab) => {
                self.sample_handle = Some(tab);

                Task::none()
            }
        }
    }

    fn as_retention_table(&self) -> String {
        let mut content = (0..self.samples.len())
            .fold(String::from("Lipid,Expected Time (s)"), |accum, i| {
                accum + &format!(",{}", i)
            });
        content.push_str("\n");

        for index in 0..self.lipid_reference.len() {
            let (time, lipid) = &self.lipid_reference[index];
            content.push_str(&lipid);
            content.push_str(&format!(",{}", time));
            for sample in &self.samples {
                let retention_time = sample
                    .lipids
                    .get(index)
                    .map_or(0.0, |peak| peak.turning_point.x());
                content.push_str(&format!(",{}", retention_time));
            }
            content.push_str("\n");
        }

        content
    }
}
