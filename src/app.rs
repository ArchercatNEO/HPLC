use std::fs;
use std::path::PathBuf;

use iced::{
    Element, Length, Point, Task,
    alignment::Horizontal,
    widget::{button, column, radio, row, scrollable, text, toggler},
};
use plotters_iced::ChartWidget;

use crate::{
    chromatography::{Chromatography, SampleType},
    cubic::Cubic,
    expandable_slider::{ExpandableSlider, Message as SliderMessage},
    reference::Reference,
    vector::*,
};

#[derive(Debug)]
pub struct App {
    lipid_reference: Vec<Reference>,
    samples: Vec<Chromatography>,
    sample_handle: Option<usize>,
    blank_handle: Option<usize>,
    dex_handle: Option<usize>,
    glucose_transformer: Option<Cubic>,
    standard_handle: Option<usize>,
    chart_start: ExpandableSlider,
    chart_end: ExpandableSlider,
    height_requirement: ExpandableSlider,
    inflection_requirement: ExpandableSlider,
    retention_time_tolerance: ExpandableSlider,
    glucose_unit_tolerance: ExpandableSlider,
    zoom_x: ExpandableSlider,
    zoom_y: ExpandableSlider,
    include_unknowns: bool,
}

impl Default for App {
    fn default() -> Self {
        let mut zoom_x = ExpandableSlider::new(0.0, 0.0, 100.0, 1.1, "Horizontal Zoom");
        let mut zoom_y = ExpandableSlider::new(0.0, 0.0, 100.0, 1.1, "Vertical Zoom");

        zoom_x.set_exponential(true);
        zoom_y.set_exponential(true);

        Self {
            lipid_reference: vec![],
            samples: vec![],
            sample_handle: None,
            blank_handle: None,
            dex_handle: None,
            glucose_transformer: None,
            standard_handle: None,
            chart_start: ExpandableSlider::new(8.5, 0.0, 60.0, 0.5, "Chart Start"),
            chart_end: ExpandableSlider::new(36.5, 0.0, 60.0, 0.5, "Chart End"),
            height_requirement: ExpandableSlider::new(0.3, 0.0, 1.0, 0.01, "Height Requirement"),
            inflection_requirement: ExpandableSlider::new(
                0.0,
                0.0,
                10.0,
                1.0,
                "Inflection Requirement",
            ),
            retention_time_tolerance: ExpandableSlider::new(
                0.2,
                0.0,
                1.0,
                0.01,
                "Retention Time Tolerance",
            ),
            glucose_unit_tolerance: ExpandableSlider::new(
                0.02,
                0.0,
                1.0,
                0.01,
                "Glucose Unit Tolerance",
            ),
            zoom_x,
            zoom_y,
            include_unknowns: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Done,
    DataFile,
    DataLoad(Vec<PathBuf>),
    ReferenceFile,
    ReferenceLoad(Vec<Reference>),
    ExportFile,
    ExportFileContent(PathBuf),
    ChartStart(SliderMessage),
    ChartEnd(SliderMessage),
    HeightRequirement(SliderMessage),
    InflectionRequirement(SliderMessage),
    RetentionTimeTolerance(SliderMessage),
    GlucoseUnitTolerance(SliderMessage),
    ZoomX(SliderMessage),
    ZoomY(SliderMessage),
    ShowUnknowns(bool),
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

        let chart_start = self.chart_start.view().map(Message::ChartStart);

        let chart_end = self.chart_end.view().map(Message::ChartEnd);

        let height_requirement = self
            .height_requirement
            .view()
            .map(Message::HeightRequirement);

        let inflection_requirement = self
            .inflection_requirement
            .view()
            .map(Message::InflectionRequirement);

        let retention_time_tolerance = self
            .retention_time_tolerance
            .view()
            .map(Message::RetentionTimeTolerance);

        let glucose_unit_tolerance = self
            .glucose_unit_tolerance
            .view()
            .map(Message::GlucoseUnitTolerance);

        let zoom_x = self.zoom_x.view().map(Message::ZoomX);

        let zoom_y = self.zoom_y.view().map(Message::ZoomY);

        let options = column![
            load_data_file,
            load_reference_file,
            export_file,
            chart_start,
            chart_end,
            height_requirement,
            inflection_requirement,
            retention_time_tolerance,
            glucose_unit_tolerance,
            zoom_x,
            zoom_y
        ]
        .width(700);

        let unknown_lipid = {
            let toggle = toggler(self.include_unknowns).on_toggle(Message::ShowUnknowns);
            let label = text("Show Unknown Lipids").align_x(Horizontal::Center);
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

        let options2 = column![unknown_lipid, sample_type].width(250);

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

            let footer = row![options, options2];
            let chart: Element<()> = ChartWidget::new(sample.clone()).width(Length::Fill).into();

            let body = row![tabs, chart.map(Message::from)];
            column![header, body, footer, table]
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
            Message::DataLoad(paths) => {
                for path in paths {
                    let mut sample = match Chromatography::from_file(&path) {
                        Some(value) => value,
                        None => continue,
                    };

                    sample.set_data_range(self.chart_start.get_value()..self.chart_end.get_value());
                    sample.set_lipid_references(self.lipid_reference.clone());
                    sample.set_include_unknowns(self.include_unknowns);
                    sample.set_height_requirement(self.height_requirement.get_value());
                    sample.set_retention_time_tolerance(self.retention_time_tolerance.get_value());
                    sample.set_glucose_unit_tolerance(self.glucose_unit_tolerance.get_value());
                    sample.set_glucose_transformer(self.glucose_transformer);
                    let zoom = Point::new(self.zoom_x.get_value(), self.zoom_y.get_value());
                    sample.set_global_zoom(zoom);
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
                        let path = handle.path();
                        let data = Reference::parse_file(&path);

                        Message::ReferenceLoad(data)
                    } else {
                        Message::Done
                    }
                })
            }
            Message::ReferenceLoad(data) => {
                self.update_parameter(&Chromatography::set_lipid_references, data.clone());
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
                if let Some(unbound) = self.chart_start.update(message) {
                    let start = f32::min(unbound, self.chart_end.get_value());
                    self.chart_start.update(SliderMessage::Value(start));

                    let range = start..self.chart_end.get_value();
                    self.update_parameter(&Chromatography::set_data_range, range);
                }

                Task::none()
            }
            Message::ChartEnd(message) => {
                if let Some(unbound) = self.chart_end.update(message) {
                    let end = f32::max(self.chart_end.get_value(), unbound);
                    self.chart_end.update(SliderMessage::Value(end));

                    let range = self.chart_start.get_value()..end;
                    self.update_parameter(&Chromatography::set_data_range, range);
                }

                Task::none()
            }
            Message::HeightRequirement(message) => {
                if let Some(value) = self.height_requirement.update(message) {
                    self.update_parameter(&Chromatography::set_height_requirement, value);
                }

                Task::none()
            }
            Message::InflectionRequirement(message) => {
                if let Some(value) = self.inflection_requirement.update(message) {
                    self.update_parameter(&Chromatography::set_inflection_requirement, value);
                }

                Task::none()
            }
            Message::RetentionTimeTolerance(message) => {
                if let Some(value) = self.retention_time_tolerance.update(message) {
                    self.update_parameter(&Chromatography::set_retention_time_tolerance, value);
                }

                Task::none()
            }
            Message::GlucoseUnitTolerance(message) => {
                if let Some(value) = self.glucose_unit_tolerance.update(message) {
                    self.update_parameter(&Chromatography::set_glucose_unit_tolerance, value);
                }

                Task::none()
            }
            Message::ShowUnknowns(show) => {
                self.include_unknowns = show;
                self.update_parameter(&Chromatography::set_include_unknowns, show);

                Task::none()
            }
            Message::ZoomX(zoom) => {
                if let Some(value) = self.zoom_x.update(zoom) {
                    let point = Point::new(value, self.zoom_y.get_value());
                    self.update_parameter(&Chromatography::set_global_zoom, point);
                }

                Task::none()
            }
            Message::ZoomY(zoom) => {
                if let Some(value) = self.zoom_y.update(zoom) {
                    let point = Point::new(self.zoom_x.get_value(), value);
                    self.update_parameter(&Chromatography::set_global_zoom, point);
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
                        }
                        SampleType::Dex => {
                            if let Some(dex_handle) = self.dex_handle {
                                self.samples[dex_handle].set_sample_type(SampleType::Data);
                            }

                            self.dex_handle = Some(handle);
                            self.glucose_transformer =
                                Some(self.samples[handle].get_glucose_transformer());
                            self.update_parameter(
                                &Chromatography::set_glucose_transformer,
                                self.glucose_transformer,
                            );
                        }
                        SampleType::Standard => {
                            if let Some(standard_handle) = self.standard_handle {
                                self.samples[standard_handle].set_sample_type(SampleType::Data);
                            }

                            self.standard_handle = Some(handle);
                            let peak = self.samples[handle].peaks[1].clone();
                            self.update_parameter(&Chromatography::set_standard_peak, Some(peak));
                        }
                    }

                    self.samples[handle].set_sample_type(sample_type);
                }

                Task::none()
            }
        }
    }

    fn update_parameter<
        TParam: Clone,
        TFun: Fn(&mut Chromatography, TParam) -> &mut Chromatography,
    >(
        &mut self,
        func: &TFun,
        value: TParam,
    ) {
        for sample in self.samples.iter_mut() {
            func(sample, value.clone());
        }

        if let Some(handle) = self.dex_handle {
            let sample = &mut self.samples[handle];

            let new_transformer = Some(sample.get_glucose_transformer());
            if self.glucose_transformer != new_transformer {
                self.glucose_transformer = new_transformer;
                for sample in self.samples.iter_mut() {
                    sample.set_glucose_transformer(new_transformer);
                }
            }
        }
    }

    fn as_retention_table(&self) -> String {
        let mut titles = String::new();
        for sample in &self.samples {
            titles.push_str(&format!(",{}", sample.title.as_ref().unwrap()));
        }

        let mut content = String::from("Retention Times\n");
        content.push_str("Lipid,Expected Time (m)");
        content.push_str(&titles);

        for (i, reference) in self.lipid_reference.iter().enumerate() {
            content.push_str("\n");
            content.push_str(&reference.name);
            content.push_str(&format!(",{}", reference.retention_time));
            for sample in &self.samples {
                let retention_time = sample
                    .lipids
                    .get(i)
                    .map_or(0.0, |peak| peak.retention_point.x());
                content.push_str(&format!(",{}", retention_time));
            }
        }

        if let Some(function) = self.glucose_transformer {
            content.push_str("\n\n");
            content.push_str("Glucose Units\n");
            content.push_str("Lipid,Expected GU (m)");
            content.push_str(&titles);

            for (i, reference) in self.lipid_reference.iter().enumerate() {
                content.push_str("\n");
                content.push_str(&reference.name);
                content.push_str(&format!(",{}", reference.glucose_units));
                for sample in &self.samples {
                    let retention_time = sample
                        .lipids
                        .get(i)
                        .map_or(0.0, |peak| function.evaluate(peak.retention_point.x()));
                    content.push_str(&format!(",{}", retention_time));
                }
            }
        }

        content.push_str("\n\n");
        content.push_str("Areas\n");
        content.push_str("Lipid");
        content.push_str(&titles);

        content.push_str("\nTotal Area");
        for sample in &self.samples {
            content.push_str(&format!(",{}", sample.total_area));
        }

        for (i, reference) in self.lipid_reference.iter().enumerate() {
            content.push('\n');
            content.push_str(&reference.name);
            for sample in &self.samples {
                let area = sample.lipids.get(i).map_or(0.0, |peak| peak.area);
                content.push_str(&format!(",{}", area));
            }
        }

        content.push_str("\n\nConcentrations");
        content.push_str("\nLipid");
        for sample in &self.samples {
            content.push_str(&format!(",{}", sample.title.as_ref().unwrap()));
        }

        for (i, reference) in self.lipid_reference.iter().enumerate() {
            content.push('\n');
            content.push_str(&reference.name);
            for sample in &self.samples {
                let concentration = sample.peaks.get(i).map_or(0.0, |peak| peak.concentration);
                content.push_str(&format!(",{}", concentration));
            }
        }

        content.push_str("\n\nUnknown Peaks");
        content.push_str("\nSample,Retention Time (m),Area\n");
        for (index, sample) in self.samples.iter().enumerate() {
            for peak in sample.peaks.iter().filter(|peak| peak.reference.is_none()) {
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
