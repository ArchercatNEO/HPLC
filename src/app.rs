use std::fs;
use std::path::PathBuf;

use iced::{
    Element, Length, Point, Task,
    alignment::{Horizontal, Vertical},
    widget::{button, column, radio, row, scrollable, slider, text, text_input, toggler},
};
use plotters_iced::ChartWidget;

use crate::{
    chromatography::{Chromatography, SampleType},
    vector::*,
};

#[derive(Clone, Debug)]
pub enum SliderMessage {
    Value(f32),
    ValueStr(String),
    Start(String),
    End(String),
    Step(String),
    Compact(bool),
}

//TODO: support exponential sliders
#[derive(Clone, Debug)]
struct SliderInfo {
    pub value: f32,
    start: f32,
    end: f32,
    step: f32,
    label: &'static str,
    pub value_str: String,
    start_str: String,
    end_str: String,
    step_str: String,
    expanded: bool,
}

impl SliderInfo {
    pub fn new(value: f32, start: f32, end: f32, step: f32, label: &'static str) -> Self {
        Self {
            value,
            start,
            end,
            step,
            label,
            value_str: value.to_string(),
            start_str: start.to_string(),
            end_str: end.to_string(),
            step_str: step.to_string(),
            expanded: false,
        }
    }

    pub fn view(&self) -> Element<Option<SliderMessage>> {
        let number_width = Length::FillPortion(2);

        let expanded = toggler(self.expanded).on_toggle(|bit| Some(SliderMessage::Compact(bit)));

        let label = text(format!("{}: ", self.label))
            .align_x(Horizontal::Right)
            .width(Length::FillPortion(2));

        let bar = slider(self.start..=self.end, self.value, |float| {
            Some(SliderMessage::Value(float))
        })
        .step(self.step)
        .width(Length::FillPortion(5));

        if self.expanded {
            let value = {
                let input = text_input(&self.value_str, &self.value_str)
                    .width(number_width)
                    .on_input(Self::wrap_parse(SliderMessage::ValueStr));

                row![label, input].align_y(Vertical::Center)
            };

            let range = {
                let label = text(": ");

                let start = text_input(&self.start_str, &self.start_str)
                    .width(number_width)
                    .on_input(Self::wrap_parse(SliderMessage::Start));

                let inequality = text("<= x <= ");

                let end = text_input(&self.end_str, &self.end_str)
                    .width(number_width)
                    .on_input(Self::wrap_parse(SliderMessage::End));

                row![label, start, inequality, end]
                    .spacing(5)
                    .align_y(Vertical::Center)
            };

            let step = {
                let label = text(": Step: ");
                let input = text_input(&self.step_str, &self.step_str)
                    .width(number_width)
                    .on_input(Self::wrap_parse(SliderMessage::Step));

                row![label, input].align_y(Vertical::Center)
            };

            let top = row![expanded, value, range, step].spacing(10);

            let el = column![top, bar];
            el.into()
        } else {
            let info = text(&self.value_str).width(number_width);

            row![expanded, label, bar, info].spacing(10).into()
        }
    }

    pub fn update(&mut self, message: SliderMessage) {
        match message {
            SliderMessage::Value(value) => {
                self.value = value;
                self.value_str = value.to_string();
            }
            SliderMessage::ValueStr(content) => {
                if let Ok(float) = content.parse::<f32>() {
                    self.value = float;
                }

                self.value_str = content;
            }
            SliderMessage::Start(content) => {
                if let Ok(float) = content.parse::<f32>() {
                    self.start = float;
                }

                self.start_str = content;
            }
            SliderMessage::End(content) => {
                if let Ok(float) = content.parse::<f32>() {
                    self.end = float;
                }

                self.end_str = content;
            }
            SliderMessage::Step(content) => {
                if let Ok(float) = content.parse::<f32>() {
                    self.step = float;
                }

                self.step_str = content;
            }
            SliderMessage::Compact(expanded) => {
                self.expanded = expanded;
            }
        }
    }

    fn wrap_parse<F: Fn(String) -> SliderMessage>(
        enum_fn: F,
    ) -> impl Fn(String) -> Option<SliderMessage> {
        move |content: String| {
            let mut numeric = true;
            for character in content.chars() {
                if !character.is_ascii_digit() && character != '.' {
                    numeric = false;
                    break;
                }
            }

            if numeric {
                Some(enum_fn(content))
            } else {
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct App {
    lipid_reference: Vec<(f32, String)>,
    samples: Vec<Chromatography>,
    sample_handle: Option<usize>,
    blank_handle: Option<usize>,
    dex_handle: Option<usize>,
    standard_handle: Option<usize>,
    chart_start: SliderInfo,
    chart_end: SliderInfo,
    height_requirement: SliderInfo,
    derivative_cone: SliderInfo,
    horizontal_deviation: SliderInfo,
    include_unknowns: bool,
    subtract_blank: bool,
    show_derivative: bool,
    global_zoom: Point,
}

impl Default for App {
    fn default() -> Self {
        let chart_start = SliderInfo::new(8.5, 0.0, 60.0, 0.5, "Chart Start");
        let chart_end = SliderInfo::new(33.5, 0.0, 60.0, 0.5, "Chart End");

        let height_requirement = SliderInfo::new(0.3, 0.0, 1.0, 0.01, "Height Requirement");
        let derivative_cone = SliderInfo::new(0.5, 0.0, 10.0, 0.1, "Derivative Cone");
        let horizontal_deviation = SliderInfo::new(0.2, 0.0, 1.0, 0.01, "Horizontal Deviation");

        Self {
            lipid_reference: vec![],
            samples: vec![],
            sample_handle: None,
            blank_handle: None,
            dex_handle: None,
            standard_handle: None,
            chart_start,
            chart_end,
            height_requirement,
            derivative_cone,
            horizontal_deviation,
            include_unknowns: false,
            subtract_blank: false,
            show_derivative: false,
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
    ChartStart(SliderMessage),
    ChartEnd(SliderMessage),
    HeightRequirement(SliderMessage),
    DerivativeCone(SliderMessage),
    HorizontalDeviation(SliderMessage),
    ShowUnknowns(bool),
    SubtractBlank(bool),
    ShowDerivate(bool),
    ZoomX(f32),
    ZoomY(f32),
    TabSwitch(usize),
    SampleTypeSelect(SampleType),
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

        let chart_start = self
            .chart_start
            .view()
            .map(|msg| msg.map_or(Message::Done, Message::ChartStart));

        let chart_end = self
            .chart_end
            .view()
            .map(|msg| msg.map_or(Message::Done, Message::ChartEnd));

        let height_requirement = self
            .height_requirement
            .view()
            .map(|msg| msg.map_or(Message::Done, Message::HeightRequirement));

        let derivative_cone = self
            .derivative_cone
            .view()
            .map(|msg| msg.map_or(Message::Done, Message::DerivativeCone));

        let horizontal_deviation = self
            .horizontal_deviation
            .view()
            .map(|msg| msg.map_or(Message::Done, Message::HorizontalDeviation));

        let zoom_x = {
            let label = text("Zoom X: ").align_x(Horizontal::Right).width(150);
            let slider = slider(0.0..=100.0, self.global_zoom.x, Message::ZoomX).width(300);
            let info = text(format!("{}%", f32::powf(1.1, self.global_zoom.x) * 100.0)).width(100);

            row![label, slider, info].spacing(10)
        };

        let zoom_y = {
            let label = text("Zoom Y: ").align_x(Horizontal::Right).width(150);
            let slider = slider(0.0..=100.0, self.global_zoom.y, Message::ZoomY).width(300);
            let info = text(format!("{}%", f32::powf(1.1, self.global_zoom.y) * 100.0)).width(100);

            row![label, slider, info].spacing(10)
        };

        let options = column![
            load_data_file,
            load_reference_file,
            export_file,
            chart_start,
            chart_end,
            height_requirement,
            derivative_cone,
            horizontal_deviation,
            zoom_x,
            zoom_y
        ]
        .width(700);

        let unknown_lipid = {
            let toggle = toggler(self.include_unknowns).on_toggle(Message::ShowUnknowns);
            let label = text("Show Unknown Lipids").align_x(Horizontal::Center);
            row![toggle, label]
        };

        let subtract_blank = {
            let toggle = toggler(self.subtract_blank).on_toggle(Message::SubtractBlank);
            let label = text("Subtract blank");

            row![toggle, label]
        };

        let show_derivative = {
            let toggle = toggler(self.show_derivative).on_toggle(Message::ShowDerivate);
            let label = text("Show derivative");

            row![toggle, label]
        };

        let sample_type = {
            let selected = self
                .sample_handle
                .map(|handle| self.samples[handle].get_sample_type());

            let header = text("Sample Type");

            let data = radio(
                "Data",
                SampleType::Data,
                selected,
                Message::SampleTypeSelect,
            );

            let blank = radio(
                "Blank",
                SampleType::Blank,
                selected,
                Message::SampleTypeSelect,
            );

            let dex = radio("Dex", SampleType::Dex, selected, Message::SampleTypeSelect);

            let standard = radio(
                "Standard",
                SampleType::Standard,
                selected,
                Message::SampleTypeSelect,
            );

            column![header, data, blank, dex, standard]
        };

        let options2 =
            column![unknown_lipid, subtract_blank, show_derivative, sample_type].width(250);

        let ui = if let Some(handle) = self.sample_handle {
            let header = text(format!("Sample {}", handle));
            let tabs = {
                let mut buttons = column![];
                for (i, sample) in self.samples.iter().enumerate() {
                    let content = format!("{}", i);
                    let label = text(content);
                    let button =
                        button(label)
                            .on_press(Message::TabSwitch(i))
                            .style(|_theme, _status| {
                                let style = button::Style::default();
                                let color = match sample.get_sample_type() {
                                    SampleType::Data => iced::color!(0x00ffff),     //cyan
                                    SampleType::Blank => iced::color!(0x00ff00),    //green
                                    SampleType::Dex => iced::color!(0xff0000),      //red
                                    SampleType::Standard => iced::color!(0xffaa00), //orange
                                };
                                style.with_background(color)
                            });
                    buttons = buttons.push(button);
                }
                scrollable(buttons)
            };

            let sample = &self.samples[handle];
            let table = sample.into_table_element().map(Message::from);

            let footer = row![options, options2, table];
            let scroll_footer = scrollable(footer).direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::default(),
            ));
            let chart: Element<()> = ChartWidget::new(sample.clone())
                .height(Length::Fill)
                .width(Length::Fill)
                .into();

            let body = row![tabs, chart.map(Message::from)];
            column![header, body, scroll_footer]
        } else {
            let footer = row![options, options2];
            let scroll_footer = scrollable(footer).direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::default(),
            ));
            column![scroll_footer]
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
                    let content = crate::parse_file(&path, crate::parse_line_as_data);
                    let mut sample = Chromatography::default();
                    sample.title = crate::parse_header(&path);
                    sample.show_derivative = self.show_derivative;
                    sample.set_data(content);
                    sample.set_data_range(self.chart_start.value..self.chart_end.value);
                    sample.set_lipid_master_table(self.lipid_reference.clone());
                    sample.set_include_unknowns(self.include_unknowns);
                    sample.set_height_requirement(self.height_requirement.value);
                    sample.set_derivative_cone(self.derivative_cone.value);
                    sample.set_horizontal_deviation(self.horizontal_deviation.value);
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
                        let data = crate::parse_file(&path, crate::parse_line_as_lipids);

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
            Message::ChartStart(message) => {
                let prev_value = self.chart_start.value;

                self.chart_start.update(message);
                self.chart_start.value = f32::min(self.chart_start.value, self.chart_end.value);
                self.chart_start.value_str = self.chart_start.value.to_string();

                if prev_value != self.chart_start.value {
                    for sample in self.samples.iter_mut() {
                        sample.set_data_range(self.chart_start.value..self.chart_end.value);
                    }
                }

                Task::none()
            }
            Message::ChartEnd(message) => {
                let prev_value = self.chart_end.value;

                self.chart_end.update(message);
                self.chart_end.value = f32::max(self.chart_end.value, self.chart_start.value);
                self.chart_end.value_str = self.chart_end.value.to_string();

                if prev_value != self.chart_end.value {
                    for sample in self.samples.iter_mut() {
                        sample.set_data_range(self.chart_end.value..self.chart_end.value);
                    }
                }

                Task::none()
            }
            Message::HeightRequirement(message) => {
                let prev_value = self.height_requirement.value;

                self.height_requirement.update(message);

                if prev_value != self.height_requirement.value {
                    for sample in self.samples.iter_mut() {
                        sample.set_height_requirement(self.height_requirement.value);
                    }
                }

                Task::none()
            }
            Message::DerivativeCone(message) => {
                let prev_value = self.derivative_cone.value;

                self.derivative_cone.update(message);

                if prev_value != self.derivative_cone.value {
                    for sample in self.samples.iter_mut() {
                        sample.set_derivative_cone(self.derivative_cone.value);
                    }
                }

                Task::none()
            }
            Message::HorizontalDeviation(message) => {
                let prev_value = self.horizontal_deviation.value;

                self.horizontal_deviation.update(message);

                if prev_value != self.horizontal_deviation.value {
                    for sample in self.samples.iter_mut() {
                        sample.set_horizontal_deviation(self.horizontal_deviation.value);
                    }
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
            Message::SubtractBlank(value) => {
                self.subtract_blank = value;
                for (i, sample) in self.samples.iter_mut().enumerate() {
                    if let Some(handle) = self.blank_handle {
                        if handle == i {
                            continue;
                        }
                    }

                    sample.set_subtract_blank(value);
                }

                Task::none()
            }
            Message::ShowDerivate(show) => {
                self.show_derivative = show;
                for sample in self.samples.iter_mut() {
                    sample.show_derivative = show;
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
            Message::SampleTypeSelect(sample_type) => {
                if let Some(handle) = self.sample_handle {
                    match self.samples[handle].get_sample_type() {
                        SampleType::Data => (),
                        SampleType::Blank => {
                            self.blank_handle = None;
                        }
                        SampleType::Dex => {
                            self.dex_handle = None;
                        }
                        SampleType::Standard => {
                            self.standard_handle = None;
                        }
                    };

                    match sample_type {
                        SampleType::Data => (),
                        SampleType::Blank => {
                            if let Some(blank_handle) = self.blank_handle {
                                self.samples[blank_handle].set_sample_type(SampleType::Data);
                            }

                            self.blank_handle = Some(handle);
                            self.samples[handle].set_subtract_blank(false);

                            let data = self.samples[handle].get_data();

                            for (i, sample) in self.samples.iter_mut().enumerate() {
                                if i == handle {
                                    continue;
                                }

                                sample.set_blank_data(Some(data.clone()));
                            }
                        }
                        SampleType::Dex => {
                            if let Some(dex_handle) = self.dex_handle {
                                self.samples[dex_handle].set_sample_type(SampleType::Data);
                            }

                            self.dex_handle = Some(handle);
                        }
                        SampleType::Standard => {
                            if let Some(standard_handle) = self.standard_handle {
                                self.samples[standard_handle].set_sample_type(SampleType::Data);
                            }

                            self.standard_handle = Some(handle);
                            let peak = self.samples[handle].peaks[0].clone();
                            for sample in self.samples.iter_mut() {
                                sample.set_standard_peak(Some(peak.clone()));
                            }
                        }
                    }

                    self.samples[handle].set_sample_type(sample_type);
                }

                Task::none()
            }
        }
    }

    fn as_retention_table(&self) -> String {
        let mut content = String::from("Retention Times\n");
        content.push_str("Lipid,Expected Time (m)");
        for i in 0..self.samples.len() {
            content.push_str(&format!(",{}", i));
        }

        for (i, lipid) in self.lipid_reference.iter().enumerate() {
            content.push_str("\n");
            content.push_str(&lipid.1);
            content.push_str(&format!(",{}", lipid.0));
            for sample in &self.samples {
                let retention_time = sample
                    .lipids
                    .get(i)
                    .map_or(0.0, |peak| peak.retention_point.x());
                content.push_str(&format!(",{}", retention_time));
            }
        }

        content.push_str("\n\nAreas");
        content.push_str("\nLipid");
        for i in 0..self.samples.len() {
            content.push_str(&format!(",{}", i));
        }

        content.push_str("\nTotal Area");
        for sample in &self.samples {
            content.push_str(&format!(",{}", sample.total_area));
        }

        for (i, lipid) in self.lipid_reference.iter().enumerate() {
            content.push('\n');
            content.push_str(&lipid.1);
            for sample in &self.samples {
                let area = sample.lipids.get(i).map_or(0.0, |peak| peak.area);
                content.push_str(&format!(",{}", area));
            }
        }

        content.push_str("\n\nConcentrations");
        content.push_str("\nLipid");
        for i in 0..self.samples.len() {
            content.push_str(&format!(",{}", i));
        }

        for (i, lipid) in self.lipid_reference.iter().enumerate() {
            content.push('\n');
            content.push_str(&lipid.1);
            for sample in &self.samples {
                let concentration = sample.peaks.get(i).map_or(0.0, |peak| peak.concentration);
                content.push_str(&format!(",{}", concentration));
            }
        }

        content.push_str("\n\nUnknown Peaks");
        content.push_str("\nSample,Retention Time (m),Area\n");
        for (index, sample) in self.samples.iter().enumerate() {
            for peak in sample.peaks.iter().filter(|peak| peak.lipid.is_none()) {
                let entry = format!("{},{},{}\n", index, peak.retention_point.x(), peak.area);
                content.push_str(&entry);
            }
        }

        if let Some(handle) = self.standard_handle {
            content.push_str("\nStandard Area\n");
            content.push_str(&format!("{}", self.samples[handle].peaks[0].area));
        }

        content.push('\n');

        content
    }
}
