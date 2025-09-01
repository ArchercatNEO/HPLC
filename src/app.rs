use std::{fs, rc::Rc};

use iced::{
    Element, Length, Point, Task,
    alignment::Horizontal,
    widget::{button, column, radio, row, scrollable, text, text_input, toggler},
};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget};
use rfd::FileHandle;

use crate::{
    chromatogram::ChromatogramState,
    chromatography::{Chromatography, SampleType},
    expandable_slider::{ExpandableSlider, Message as SliderMessage},
    peak::{Peak, PeakType},
    reference::Reference,
    spline::Spline,
    vector::*,
};

#[derive(Debug)]
pub struct App {
    lipid_reference: Rc<[Reference]>,
    samples: Vec<Chromatography>,
    sample_handle: Option<usize>,
    blank_handle: Option<usize>,
    dex_handle: Option<usize>,
    glucose_transformer: Option<Spline>,
    standard_handle: Option<usize>,
    injected_volume: f64,
    injected_volume_str: String,
    sample_dilution: f64,
    sample_dilution_str: String,
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
            lipid_reference: Rc::default(),
            samples: vec![],
            sample_handle: None,
            blank_handle: None,
            dex_handle: None,
            glucose_transformer: None,
            standard_handle: None,
            injected_volume: 20.0,
            injected_volume_str: "20.0".to_string(),
            sample_dilution: 40.0,
            sample_dilution_str: "40.0".to_string(),
            chart_start: ExpandableSlider::new(8.5, 0.0, 60.0, 0.5, "Chart Start"),
            chart_end: ExpandableSlider::new(36.5, 0.0, 60.0, 0.5, "Chart End"),
            height_requirement: ExpandableSlider::new(0.3, 0.0, 1.0, 0.01, "Height Requirement"),
            inflection_requirement: ExpandableSlider::new(
                10.0,
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
    RequestSamplePaths,
    LoadSampleFiles(Vec<FileHandle>),
    RequestReferencePath,
    LoadRefereceFile(FileHandle),
    RequestExportCsvPath,
    ExportCsvFile(FileHandle),
    RequestExportProfilesPath,
    ExportProfilesDirectory(FileHandle),
    ChartStart(SliderMessage),
    ChartEnd(SliderMessage),
    HeightRequirement(SliderMessage),
    InflectionRequirement(SliderMessage),
    RetentionTimeTolerance(SliderMessage),
    GlucoseUnitTolerance(SliderMessage),
    ZoomX(SliderMessage),
    ZoomY(SliderMessage),
    ShowUnknowns(bool),
    SampleTypeSelect(SampleType),
    InjectedVolume(String),
    SampleDilution(String),
    TabSwitch(usize),
}

impl From<()> for Message {
    fn from(_value: ()) -> Self {
        Message::Done
    }
}

impl App {
    pub fn view(&self) -> Element<Message> {
        let load_data_file = button("Load Raw Data File").on_press(Message::RequestSamplePaths);

        let load_reference_file =
            button("Load Lipid Reference File").on_press(Message::RequestReferencePath);

        let export_file = button("Export Table").on_press(Message::RequestExportCsvPath);

        let export_profiles =
            button("Export Profiles").on_press(Message::RequestExportProfilesPath);

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
            export_profiles,
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

        let warnings = {
            let mut content = String::new();
            if self.dex_handle.is_none() {
                content += "Dex not set. Cannot calculate GU.\n";
            }
            if self.standard_handle.is_none() {
                content += "Standard not set. Cannot calculate concentration.";
            }

            text(content).color(iced::color!(0xff0000))
        };

        let injected_volume = {
            let label = text("Vinjection (µl): ");
            let input =
                text_input("20.0", &self.injected_volume_str).on_input(Message::InjectedVolume);
            row![label, input]
        };

        let sample_dilution = {
            let label = text("Sample Dilution: ");
            let input =
                text_input("40.0", &self.sample_dilution_str).on_input(Message::SampleDilution);
            row![label, input]
        };

        let options2 = column![
            unknown_lipid,
            sample_type,
            warnings,
            injected_volume,
            sample_dilution
        ]
        .width(250);

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

            let multiplier = match self.standard_handle {
                Some(handle) => {
                    1000.0
                        * (1.0 / self.injected_volume)
                        * self.sample_dilution
                        * 0.0025
                        * (1.0 / self.samples[handle].peaks[0].area)
                }
                None => 0.0,
            };

            let sample = &self.samples[handle];
            let table = sample.into_table_element(multiplier).map(Message::from);

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
            Message::RequestSamplePaths => {
                let task = rfd::AsyncFileDialog::new()
                    .add_filter("any", &["*"])
                    .add_filter("text", &["arw", "csv", "tsv", "txt"])
                    .pick_files();

                Task::perform(task, |maybe_handles| match maybe_handles {
                    Some(handles) => Message::LoadSampleFiles(handles),
                    None => Message::Done,
                })
            }
            Message::LoadSampleFiles(handles) => {
                for handle in handles {
                    let path = handle.path();
                    let mut sample = match Chromatography::from_file(&path) {
                        Some(value) => value,
                        None => continue,
                    };

                    sample.set_data_range(
                        &(self.chart_start.get_value()..self.chart_end.get_value()),
                    );
                    sample.set_lipid_references(Rc::clone(&self.lipid_reference));
                    sample.set_include_unknowns(&self.include_unknowns);
                    sample.set_height_requirement(&self.height_requirement.get_value());
                    sample.set_retention_time_tolerance(&self.retention_time_tolerance.get_value());
                    sample.set_glucose_unit_tolerance(&self.glucose_unit_tolerance.get_value());
                    sample.set_glucose_transformer(&self.glucose_transformer);
                    let zoom = Point::new(self.zoom_x.get_value(), self.zoom_y.get_value());
                    sample.set_global_zoom(&zoom);
                    self.samples.push(sample);
                }

                self.sample_handle = Some(self.samples.len() - 1);
                Task::none()
            }
            Message::RequestReferencePath => {
                let task = rfd::AsyncFileDialog::new()
                    .add_filter("any", &["*"])
                    .add_filter("text", &["arw", "csv", "tsv", "txt"])
                    .pick_file();

                Task::perform(task, |maybe_handle| match maybe_handle {
                    Some(handle) => Message::LoadRefereceFile(handle),
                    None => Message::Done,
                })
            }
            Message::LoadRefereceFile(handle) => {
                let path = handle.path();
                let reference = Reference::parse_file(&path);
                self.lipid_reference = Rc::from(reference.as_slice());

                for sample in self.samples.iter_mut() {
                    sample.set_lipid_references(Rc::clone(&self.lipid_reference));
                }

                Task::none()
            }
            Message::RequestExportCsvPath => {
                let task = rfd::AsyncFileDialog::new()
                    .set_file_name("table.csv")
                    .save_file();

                Task::perform(task, |maybe_handle| match maybe_handle {
                    Some(handle) => Message::ExportCsvFile(handle),
                    None => Message::Done,
                })
            }
            Message::ExportCsvFile(handle) => {
                let content = self.as_retention_table();
                fs::write(handle.path(), content).expect("Cannot write there");

                Task::none()
            }
            Message::RequestExportProfilesPath => {
                let task = rfd::AsyncFileDialog::new().pick_folder();

                Task::perform(task, |maybe_handle| match maybe_handle {
                    Some(handle) => Message::ExportProfilesDirectory(handle),
                    None => Message::Done,
                })
            }
            Message::ExportProfilesDirectory(handle) => {
                let root = handle.path();

                for sample in &self.samples {
                    let mut path = root.join(&sample.file_name);
                    path.set_extension("svg");

                    let image = SVGBackend::new(&path, (1980, 1080)).into_drawing_area();
                    image.fill(&WHITE).expect("failed");

                    let builder = ChartBuilder::on(&image);
                    sample.build_chart(&ChromatogramState::default(), builder);
                }

                Task::none()
            }
            Message::ChartStart(message) => {
                if let Some(unbound) = self.chart_start.update(message) {
                    let start = f64::min(unbound, self.chart_end.get_value());
                    self.chart_start.update(SliderMessage::Value(start));

                    let range = start..self.chart_end.get_value();
                    for sample in self.samples.iter_mut() {
                        sample.set_data_range(&range);
                    }
                }

                Task::none()
            }
            Message::ChartEnd(message) => {
                if let Some(unbound) = self.chart_end.update(message) {
                    let end = f64::max(self.chart_end.get_value(), unbound);
                    self.chart_end.update(SliderMessage::Value(end));

                    let range = self.chart_start.get_value()..end;
                    for sample in self.samples.iter_mut() {
                        sample.set_data_range(&range);
                    }
                }

                Task::none()
            }
            Message::HeightRequirement(message) => {
                if let Some(value) = self.height_requirement.update(message) {
                    for sample in self.samples.iter_mut() {
                        sample.set_height_requirement(&value);
                    }
                }

                Task::none()
            }
            Message::InflectionRequirement(message) => {
                if let Some(value) = self.inflection_requirement.update(message) {
                    for sample in self.samples.iter_mut() {
                        sample.set_inflection_requirement(&value);
                    }
                }

                Task::none()
            }
            Message::RetentionTimeTolerance(message) => {
                if let Some(value) = self.retention_time_tolerance.update(message) {
                    for sample in self.samples.iter_mut() {
                        sample.set_retention_time_tolerance(&value);
                    }
                }

                Task::none()
            }
            Message::GlucoseUnitTolerance(message) => {
                if let Some(value) = self.glucose_unit_tolerance.update(message) {
                    for sample in self.samples.iter_mut() {
                        sample.set_glucose_unit_tolerance(&value);
                    }
                }

                Task::none()
            }
            Message::ShowUnknowns(show) => {
                self.include_unknowns = show;
                for sample in self.samples.iter_mut() {
                    sample.set_include_unknowns(&show);
                }

                Task::none()
            }
            Message::ZoomX(zoom) => {
                if let Some(value) = self.zoom_x.update(zoom) {
                    let point = Point::new(value, self.zoom_y.get_value());
                    for sample in self.samples.iter_mut() {
                        sample.set_global_zoom(&point);
                    }
                }

                Task::none()
            }
            Message::ZoomY(zoom) => {
                if let Some(value) = self.zoom_y.update(zoom) {
                    let point = Point::new(self.zoom_x.get_value(), value);
                    for sample in self.samples.iter_mut() {
                        sample.set_global_zoom(&point);
                    }
                }

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
                            for sample in self.samples.iter_mut() {
                                sample.set_glucose_transformer(&None);
                            }

                            self.dex_handle = None;
                        }
                        SampleType::Standard => {
                            self.standard_handle = None;
                        }
                    };

                    self.samples[handle].set_sample_type(&sample_type);

                    match sample_type {
                        SampleType::Data => (),
                        SampleType::Blank => {
                            self.blank_handle = Some(handle);
                        }
                        SampleType::Dex => {
                            self.dex_handle = Some(handle);
                            self.glucose_transformer =
                                self.samples[handle].get_glucose_transformer();
                            for sample in self.samples.iter_mut() {
                                sample.set_glucose_transformer(&self.glucose_transformer);
                            }
                        }
                        SampleType::Standard => {
                            self.standard_handle = Some(handle);
                        }
                    }
                }

                Task::none()
            }
            Message::InjectedVolume(input) => {
                for character in input.chars() {
                    if !character.is_ascii_digit() && character != '.' {
                        return Task::none();
                    }
                }

                if let Ok(value) = input.parse::<f64>() {
                    self.injected_volume = value;
                }
                self.injected_volume_str = input;

                Task::none()
            }
            Message::SampleDilution(input) => {
                for character in input.chars() {
                    if !character.is_ascii_digit() && character != '.' {
                        return Task::none();
                    }
                }

                if let Ok(value) = input.parse::<f64>() {
                    self.sample_dilution = value;
                }
                self.sample_dilution_str = input;

                Task::none()
            }
            Message::TabSwitch(tab) => {
                self.sample_handle = Some(tab);

                Task::none()
            }
        }
    }

    fn as_retention_table(&self) -> String {
        let mut titles = String::new();
        for sample in &self.samples {
            titles.push_str(&format!(",{}", &sample.title));
        }

        let mut content = String::from("Retention Times\n");
        content.push_str("Lipid,Expected Time (m)");
        content.push_str(&titles);

        let zero_or_rt = |peak: &Peak| match &peak.peak_type {
            PeakType::Missing(_) => ",".to_string(),
            _ => {
                format!(",{}", peak.retention_point.x())
            }
        };

        for (i, reference) in self.lipid_reference.iter().enumerate() {
            if let Some(time) = reference.retention_time {
                content.push_str("\n");
                content.push_str(reference.name.as_ref().map_or("[Unnamed]", |inner| &inner));
                content.push_str(&format!(",{}", time));
                for sample in &self.samples {
                    let retention_time = sample.lipids.get(i).map_or(",".to_string(), zero_or_rt);
                    content.push_str(&retention_time);
                }
            }
        }

        if self.glucose_transformer.is_some() {
            content.push_str("\n\n");
            content.push_str("Glucose Units\n");
            content.push_str("Lipid,Expected GU");
            content.push_str(&titles);

            let zero_or_gu = |peak: &Peak| match &peak.peak_type {
                PeakType::Missing(_) => ",".to_string(),
                _ => format!(",{:.3}", peak.gu.unwrap()),
            };

            for (i, reference) in self.lipid_reference.iter().enumerate() {
                content.push_str("\n");
                content.push_str(reference.name.as_ref().map_or("[Unnamed]", |inner| &inner));
                if let Some(gu) = reference.glucose_units {
                    content.push_str(&format!(",{}", gu));
                } else {
                    content.push_str(",");
                }
                for sample in &self.samples {
                    let retention_time = sample.lipids.get(i).map_or(",".to_string(), zero_or_gu);
                    content.push_str(&retention_time);
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
            content.push_str(reference.name.as_ref().map_or("[Unnamed]", |inner| &inner));
            for sample in &self.samples {
                let area = sample.lipids.get(i).map_or(0.0, |peak| peak.area);
                content.push_str(&format!(",{}", area));
            }
        }

        if let Some(handle) = self.standard_handle {
            let multiplier = 1000.0
                * (1.0 / self.injected_volume)
                * self.sample_dilution
                * 0.0025
                * (1.0 / self.samples[handle].peaks[0].area);

            content.push_str("\n\nConcentrations");
            content.push_str("\nLipid");
            content.push_str(&titles);

            content.push_str("\nTotal Concentration");
            for sample in &self.samples {
                content.push_str(&format!(",{}", sample.total_area * multiplier));
            }

            for (i, reference) in self.lipid_reference.iter().enumerate() {
                content.push('\n');
                content.push_str(reference.name.as_ref().map_or("[Unnamed]", |inner| &inner));
                for sample in &self.samples {
                    let concentration = sample
                        .lipids
                        .get(i)
                        .map_or(0.0, |peak| peak.area * multiplier);
                    content.push_str(&format!(",{}", concentration));
                }
            }
        }

        content.push_str("\n\nDiscovered Peaks");

        content.push_str("\n\nRetention Times");
        content.push_str("\nPeak");
        content.push_str(&titles);

        let mut peak_sets: Vec<_> = self
            .samples
            .iter()
            .map(|sample| sample.peaks.iter())
            .collect();

        let mut index = 0;
        let mut exhausted = false;
        while !exhausted {
            exhausted = true;
            content.push_str(&format!("\n{}", index));

            for peaks in &mut peak_sets {
                if let Some(peak) = peaks.next() {
                    exhausted = false;
                    content.push_str(&format!(",{}", peak.retention_point.x()));
                } else {
                    content.push_str(",");
                }
            }

            index += 1;
        }

        content.push_str("\n\nUnlabelled Retention Times");
        content.push_str("\nPeak");
        content.push_str(&titles);

        let mut peak_sets: Vec<_> = self
            .samples
            .iter()
            .map(|sample| {
                sample
                    .peaks
                    .iter()
                    .filter(|peak| peak.peak_type == PeakType::Unknown)
            })
            .collect();

        let mut index = 0;
        let mut exhausted = false;
        while !exhausted {
            exhausted = true;
            content.push_str(&format!("\n{}", index));

            for peaks in &mut peak_sets {
                if let Some(peak) = peaks.next() {
                    exhausted = false;
                    content.push_str(&format!(",{}", peak.retention_point.x()));
                } else {
                    content.push_str(",");
                }
            }

            index += 1;
        }

        if let Some(function) = &self.glucose_transformer {
            content.push_str("\n\nGlucose Units");
            content.push_str("\nPeak");
            content.push_str(&titles);

            let mut peak_sets: Vec<_> = self
                .samples
                .iter()
                .map(|sample| sample.peaks.iter())
                .collect();

            let mut index = 0;
            let mut exhausted = false;
            while !exhausted {
                exhausted = true;
                content.push_str(&format!("\n{}", index));

                for peaks in &mut peak_sets {
                    if let Some(peak) = peaks.next() {
                        exhausted = false;
                        let gu = match peak.gu {
                            Some(gu) => format!(",{:.3}", gu),
                            None => format!(",OOR"),
                        };
                        content.push_str(&gu);
                    } else {
                        content.push_str(",");
                    }
                }

                index += 1;
            }

            content.push_str("\n\nUnlabelled Glucose");
            content.push_str("\nPeak");
            content.push_str(&titles);

            let mut peak_sets: Vec<_> = self
                .samples
                .iter()
                .map(|sample| {
                    sample
                        .peaks
                        .iter()
                        .filter(|peak| peak.peak_type == PeakType::Unknown)
                })
                .collect();

            let mut index = 0;
            let mut exhausted = false;
            while !exhausted {
                exhausted = true;
                content.push_str(&format!("\n{}", index));

                for peaks in &mut peak_sets {
                    if let Some(peak) = peaks.next() {
                        exhausted = false;
                        content.push_str(&format!(",{:?}", peak.gu));
                    } else {
                        content.push_str(",");
                    }
                }

                index += 1;
            }
        } else {
            content.push_str("\n\nDex not set. Cannot calculate GU");
        }

        if let Some(handle) = self.standard_handle {
            content.push_str("\nStandard Area\n");
            content.push_str(&format!("{}", self.samples[handle].peaks[0].area));
        }

        content.push('\n');

        content
    }
}
