use std::rc::Rc;

use iced::{
    Element, Length, Point, Subscription, Task,
    alignment::Horizontal,
    widget::{button, column, radio, row, scrollable, text, text_input, toggler},
    window::{self, Settings, events},
};
use plotters_iced::ChartWidget;
use rfd::FileHandle;

use crate::{
    chromatography::{Chromatography, SampleType},
    expandable_slider::{ExpandableSlider, Message as SliderMessage},
    exporter::{self, Exporter},
    reference::Reference,
    spline::Spline,
};

//TODO implement docking
#[derive(Debug)]
pub struct App {
    main_window: window::Id,
    lipid_reference: Rc<[Reference]>,
    samples: Vec<Chromatography>,
    sample_handle: Option<usize>,
    blank_handle: Option<usize>,
    dex_handle: Option<usize>,
    standard_handle: Option<usize>,
    glucose_transformer: Option<Spline>,
    concentration_multiplier: Option<f64>,
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
    exporter: Exporter,
}

#[derive(Clone, Debug)]
pub enum Message {
    None,
    CloseWindow(window::Id),
    RequestSamplePaths,
    LoadSampleFiles(Vec<FileHandle>),
    RequestReferencePath,
    LoadRefereceFile(FileHandle),
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
    ExporterMessage(exporter::Message),
}

impl From<()> for Message {
    fn from(_value: ()) -> Self {
        Message::None
    }
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let settings = Settings::default();
        let (id, task) = window::open(settings);

        // TODO load configuration from settings
        let chart_start = ExpandableSlider::new(8.5, 0.0, 60.0, 0.5, "Chart Start");
        let chart_end = ExpandableSlider::new(36.5, 0.0, 60.0, 0.5, "Chart End");
        let height_requirement = ExpandableSlider::new(0.3, 0.0, 1.0, 0.01, "Height Requirement");
        let inflection_requirement =
            ExpandableSlider::new(10.0, 0.0, 10.0, 1.0, "Inflection Requirement");
        let retention_time_tolerance =
            ExpandableSlider::new(0.2, 0.0, 1.0, 0.01, "Retention Time Tolerance");

        let glucose_unit_tolerance =
            ExpandableSlider::new(0.02, 0.0, 1.0, 0.01, "Glucose Unit Tolerance");

        let mut zoom_x = ExpandableSlider::new(0.0, 0.0, 100.0, 1.1, "Horizontal Zoom");
        let mut zoom_y = ExpandableSlider::new(0.0, 0.0, 100.0, 1.1, "Vertical Zoom");

        zoom_x.set_exponential(true);
        zoom_y.set_exponential(true);

        let app = Self {
            main_window: id,
            lipid_reference: Rc::default(),
            samples: Vec::default(),
            sample_handle: None,
            blank_handle: None,
            dex_handle: None,
            standard_handle: None,
            glucose_transformer: None,
            concentration_multiplier: None,
            injected_volume: 50.0,
            injected_volume_str: "50.0".to_string(),
            sample_dilution: 40.0,
            sample_dilution_str: "40.0".to_string(),
            chart_start,
            chart_end,
            height_requirement,
            inflection_requirement,
            retention_time_tolerance,
            glucose_unit_tolerance,
            zoom_x,
            zoom_y,
            include_unknowns: false,
            exporter: Exporter::default(),
        };

        (app, task.map(|_| Message::None))
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if self.exporter.owns_window(window_id) {
            return self
                .exporter
                .view(
                    window_id,
                    &self.samples,
                    self.dex_handle.is_some(),
                    self.standard_handle.is_some(),
                )
                .map(Message::ExporterMessage);
        }

        let load_data_file = button("Load Raw Data File").on_press(Message::RequestSamplePaths);

        let load_reference_file =
            button("Load Lipid Reference File").on_press(Message::RequestReferencePath);

        let export_file = self
            .exporter
            .external_csv_view()
            .map(Message::ExporterMessage);

        let export_profiles = self
            .exporter
            .external_profile_view()
            .map(Message::ExporterMessage);

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
                text_input("50.0", &self.injected_volume_str).on_input(Message::InjectedVolume);
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

            let sample = &self.samples[handle];
            let table = sample
                .into_table_element(self.concentration_multiplier.unwrap_or(0.0))
                .map(Message::from);

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
            Message::None => Task::none(),
            Message::CloseWindow(id) => {
                if id == self.main_window {
                    iced::exit().map(|_: ()| Message::None)
                } else {
                    window::close(id).map(|_: ()| Message::None)
                }
            }
            Message::RequestSamplePaths => {
                let task = rfd::AsyncFileDialog::new()
                    .add_filter("any", &["*"])
                    .add_filter("text", &["arw", "csv", "tsv", "txt"])
                    .pick_files();

                Task::perform(task, |maybe_handles| match maybe_handles {
                    Some(handles) => Message::LoadSampleFiles(handles),
                    None => Message::None,
                })
            }
            Message::LoadSampleFiles(handles) => {
                for handle in handles {
                    let path = handle.path();
                    let mut sample = match Chromatography::from_file(&path) {
                        Some(value) => value,
                        None => continue,
                    };

                    let range = self.chart_start.get_value()..self.chart_end.get_value();
                    sample.set_data_range(&range);
                    sample.set_lipid_references(Rc::clone(&self.lipid_reference));
                    sample.set_include_unknowns(&self.include_unknowns);
                    sample.set_height_requirement(&self.height_requirement.get_value());
                    sample.set_inflection_requirement(&self.inflection_requirement.get_value());
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
                    None => Message::None,
                })
            }
            Message::LoadRefereceFile(handle) => {
                let path = handle.path();
                let reference = Reference::parse_file(&path);
                self.lipid_reference = Rc::from(reference.as_slice());
                self.exporter
                    .set_lipid_references(Rc::clone(&self.lipid_reference));

                for sample in self.samples.iter_mut() {
                    sample.set_lipid_references(Rc::clone(&self.lipid_reference));
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
                            self.exporter
                                .set_glucose_spline(self.glucose_transformer.as_ref());
                            for sample in self.samples.iter_mut() {
                                sample.set_glucose_transformer(&self.glucose_transformer);
                            }
                        }
                        SampleType::Standard => {
                            let area = self.samples[handle].get_unqualified_components()[0].area;
                            self.standard_handle = Some(handle);
                            self.concentration_multiplier = Some(
                                1000.0
                                    * (1.0 / self.injected_volume)
                                    * self.sample_dilution
                                    * 0.0025
                                    * (1.0 / area),
                            );

                            self.exporter.set_concentration_multiplier(
                                Some(area),
                                self.concentration_multiplier,
                            );
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
                    self.concentration_multiplier = self.standard_handle.map(|handle| {
                        1000.0
                            * (1.0 / self.injected_volume)
                            * self.sample_dilution
                            * 0.0025
                            * (1.0 / self.samples[handle].get_unqualified_components()[0].area)
                    });

                    let area = self
                        .standard_handle
                        .map(|handle| self.samples[handle].get_unqualified_components()[0].area);

                    self.exporter
                        .set_concentration_multiplier(area, self.concentration_multiplier);
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
                    self.concentration_multiplier = self.standard_handle.map(|handle| {
                        1000.0
                            * (1.0 / self.injected_volume)
                            * self.sample_dilution
                            * 0.0025
                            * (1.0 / self.samples[handle].get_unqualified_components()[0].area)
                    });

                    let area = self
                        .standard_handle
                        .map(|handle| self.samples[handle].get_unqualified_components()[0].area);

                    self.exporter
                        .set_concentration_multiplier(area, self.concentration_multiplier);
                }
                self.sample_dilution_str = input;

                Task::none()
            }
            Message::TabSwitch(tab) => {
                self.sample_handle = Some(tab);

                Task::none()
            }
            Message::ExporterMessage(msg) => self
                .exporter
                .update(msg, &self.samples)
                .map(Message::ExporterMessage),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        events().map(|(id, event)| {
            match event {
                //TODO: difference?
                window::Event::Closed => Message::CloseWindow(id),
                window::Event::CloseRequested => Message::CloseWindow(id),
                // TODO: use
                window::Event::FileHovered(_) => Message::None,
                window::Event::FileDropped(_) => Message::None,
                window::Event::FilesHoveredLeft => Message::None,
                _ => Message::None,
            }
        })
    }
}
