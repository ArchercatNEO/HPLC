use iced::{
    Element, Length, Task,
    alignment::{Horizontal, Vertical},
    widget::{self, Column, Space, button, checkbox, column, container, row, scrollable, text},
    window::{self, Settings},
};
use plotters::prelude::*;
use plotters_iced::Chart;
use rfd::FileHandle;

use std::{fs, rc::Rc};

use crate::{
    chromatogram::ChromatogramState,
    chromatography::{Chromatography, ComponentFilter},
    component::Component,
    reference::Reference,
    spline::Spline,
};

#[derive(Clone, Debug)]
pub enum Message {
    None,
    OpenCsvDialog,
    QueryTargetFile,
    QueryTargetDirectory,
    TargetFile(FileHandle),
    TargetDirectory(FileHandle),
    RetentionTime(bool),
    GlucoseUnits(bool),
    Area(bool),
    Concentration(bool),
    Transpose(bool),
    IncludeExpected(bool),
    IncludeExisting(bool),
}

#[derive(Debug, Default)]
pub struct Exporter {
    // Owned windows.
    database_id: Option<window::Id>,
    profiles_id: Option<window::Id>,

    // Other state.
    references: Rc<[Reference]>,
    glucose_spline: Option<Spline>,
    standard_area: Option<f64>,
    conc_multiplier: Option<f64>,

    //User-defined state in order of appearance
    // Content.
    retention_time: bool,
    glucose_units: bool,
    area: bool,
    concentration: bool,

    include_expected: bool,
    include_existing: bool,

    // Other Settings.
    transpose: bool,
}

impl Exporter {
    pub fn external_csv_view(&self) -> Element<'_, Message> {
        button("Export as CSV")
            .on_press(Message::OpenCsvDialog)
            .into()
    }

    pub fn external_profile_view(&self) -> Element<'_, Message> {
        button("Export as profile")
            .on_press(Message::QueryTargetDirectory)
            .into()
    }

    pub fn view(
        &self,
        window_id: window::Id,
        samples: &[Chromatography],
        enable_gu: bool,
        enable_concentration: bool,
    ) -> Element<'_, Message> {
        if Some(window_id) == self.database_id {
            self.csv_view(samples, enable_gu, enable_concentration)
        } else {
            self.profile_view(samples)
        }
    }

    fn csv_view(
        &self,
        samples: &[Chromatography],
        enable_gu: bool,
        enable_concentration: bool,
    ) -> Element<'_, Message> {
        let retention_time =
            checkbox("Retention Time", self.retention_time).on_toggle(Message::RetentionTime);

        let glucose_units: Element<'_, Message> = if enable_gu {
            checkbox("Glucose Units", self.glucose_units)
                .on_toggle(Message::GlucoseUnits)
                .into()
        } else {
            let disable: Option<fn(bool) -> Message> = None;
            let toggle = checkbox("Glucose Units", self.glucose_units).on_toggle_maybe(disable);
            let warning = text("Dex not set! Cannot calculate Glucose Units")
                .color(iced::Color::new(1.0, 0.0, 0.0, 1.0));
            row![toggle, warning].into()
        };

        let area = checkbox("Area", self.area).on_toggle(Message::Area);

        let concentration: Element<'_, Message> = if enable_concentration {
            checkbox("Concentration", self.concentration)
                .on_toggle(Message::Concentration)
                .into()
        } else {
            let disable: Option<fn(bool) -> Message> = None;
            let toggle = checkbox("Concentration", self.concentration).on_toggle_maybe(disable);
            let warning = text("Standard not set! Cannot calculate concentration")
                .color(iced::Color::new(1.0, 0.0, 0.0, 1.0));
            row![toggle, warning].into()
        };

        let transpose = checkbox("Transpose", self.transpose).on_toggle(Message::Transpose);

        let include_expected =
            checkbox("Include Expected", self.include_expected).on_toggle(Message::IncludeExpected);

        let include_existing =
            checkbox("Include Existing", self.include_existing).on_toggle(Message::IncludeExisting);

        let preview = {
            let builder = TableBuilderElement::new(self.references.clone(), samples);
            let element = self.export_table(builder);

            let direction = widget::scrollable::Direction::Both {
                vertical: widget::scrollable::Scrollbar::new(),
                horizontal: widget::scrollable::Scrollbar::new(),
            };

            scrollable(element.map(|_| Message::None))
                .direction(direction)
                .width(700)
                .height(300)
        };

        let export = button("Export").on_press(Message::QueryTargetFile);

        let content = widget::column![
            retention_time,
            glucose_units,
            area,
            concentration,
            transpose,
            include_expected,
            include_existing,
            preview,
            export
        ];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    fn profile_view(&self, _samples: &[Chromatography]) -> Element<'_, Message> {
        button("Export Profiles")
            .on_press(Message::QueryTargetDirectory)
            .into()
    }

    pub fn update(&mut self, msg: Message, samples: &[Chromatography]) -> Task<Message> {
        match msg {
            Message::None => Task::none(),
            Message::OpenCsvDialog => {
                let (id, task) = window::open(Settings::default());
                self.database_id = Some(id);
                task.map(|_| Message::None)
            }
            Message::QueryTargetFile => {
                let task = rfd::AsyncFileDialog::new()
                    .set_file_name("table.csv")
                    .save_file();

                Task::perform(task, |maybe_handle| match maybe_handle {
                    None => Message::None,
                    Some(handle) => Message::TargetFile(handle),
                })
            }
            Message::QueryTargetDirectory => {
                let task = rfd::AsyncFileDialog::new().pick_folder();

                Task::perform(task, |maybe_handle| match maybe_handle {
                    None => Message::None,
                    Some(handle) => Message::TargetDirectory(handle),
                })
            }
            Message::TargetFile(file_handle) => {
                let builder = TableBuilderCsv::new(self.references.clone(), samples);
                let mut content = self.export_table(builder);
                if let Some(area) = self.standard_area {
                    content.push_str(&format!("Standard Area: {:.3}", area));
                }
                let path = file_handle.path();
                let _ = fs::write(path, content);
                Task::none()
            }
            Message::TargetDirectory(file_handle) => {
                let root = file_handle.path();

                for sample in samples {
                    let mut path = root.join(&sample.file_name);
                    path.set_extension("svg");

                    let image = SVGBackend::new(&path, (1980, 1080)).into_drawing_area();
                    image.fill(&WHITE).expect("failed");

                    let builder = ChartBuilder::on(&image);
                    sample.build_chart(&ChromatogramState::default(), builder);
                }

                Task::none()
            }
            Message::RetentionTime(enable) => {
                self.retention_time = enable;
                Task::none()
            }
            Message::GlucoseUnits(enable) => {
                self.glucose_units = enable;
                Task::none()
            }
            Message::Area(enable) => {
                self.area = enable;
                Task::none()
            }
            Message::Concentration(enable) => {
                self.concentration = enable;
                Task::none()
            }
            Message::Transpose(enable) => {
                self.transpose = enable;
                Task::none()
            }
            Message::IncludeExpected(enable) => {
                self.include_expected = enable;
                Task::none()
            }
            Message::IncludeExisting(enable) => {
                self.include_existing = enable;
                Task::none()
            }
        }
    }

    pub fn owns_window(&self, window_id: window::Id) -> bool {
        Some(window_id) == self.database_id || Some(window_id) == self.profiles_id
    }

    pub fn set_lipid_references(&mut self, references: Rc<[Reference]>) {
        self.references = references;
    }

    pub fn set_glucose_spline(&mut self, spline: Option<&Spline>) {
        self.glucose_spline = spline.cloned();
    }

    pub fn set_concentration_multiplier(&mut self, area: Option<f64>, multiplier: Option<f64>) {
        self.standard_area = area;
        self.conc_multiplier = multiplier;
    }

    fn export_table<TOut, B: TableBuilder<TOut>>(&self, mut builder: B) -> TOut {
        builder.set_transpose(self.transpose);

        if self.retention_time {
            if self.include_expected {
                builder.set_reference_additional("Expected Time", &Reference::get_expected_rt);
                builder.build_expected_section("Retention Time", &Component::get_experimental_rt);
            }

            if self.include_existing {
                builder.build_existing_section("Retention Time", &Component::get_experimental_rt);
            }
        }

        if self.glucose_units {
            if self.include_expected {
                let copy = self.glucose_spline.clone();
                builder.set_reference_additional("Expected GU", move |reference: &Reference| {
                    reference.get_expected_gu(copy.as_ref())
                });
                builder.build_expected_section("Glucose Units", |component| {
                    component.get_experimental_gu(self.glucose_spline.as_ref())
                });
            }

            if self.include_existing {
                builder.build_existing_section("Glucose Units", |component| {
                    component.get_experimental_gu(self.glucose_spline.as_ref())
                });
            }
        }

        if self.area {
            if self.include_expected {
                builder.set_sample_additional("Total Area", |sample: &Chromatography| {
                    Some(sample.total_area)
                });

                builder.build_expected_section("Area", &Component::get_area);
            }

            if self.include_existing {
                builder.build_existing_section("Area", &Component::get_area);
            }
        }

        if self.concentration {
            if let Some(factor) = self.conc_multiplier {
                if self.include_expected {
                    builder.set_sample_additional(
                        "Total Concentration",
                        move |sample: &Chromatography| Some(sample.total_area * factor),
                    );
                    builder.build_expected_section("Concentration", |component| {
                        component.get_area().map(|area| area * factor)
                    });
                }

                if self.include_existing {
                    builder.build_existing_section("Concentration", |component| {
                        component.get_area().map(|area| area * factor)
                    });
                }
            } else {
                println!("Attempted to export concentrations without a standard set.");
            }
        }

        builder.build()
    }
}

// There are 8 export variants depending on filter settings
// * None -> Why?
// * Unknown only -> simple title vs component index
// * Located only -> ??????
// * Unknown + Located (existing) -> simple title vs component index
// * Reference only -> useless, that's what the reference is for
// * Reference + Unknown -> title vs component index but the reference are used as barriers
// * Reference + Located (expected) -> title vs reference
// * Reference + Located + Unknown -> very complex

// They can be reduced to:
// * Existing only (bool include located)
// * Expected only
// * Unknown only with barriers
// * Existing only with barriers
trait TableBuilder<T> {
    fn set_reference_additional<F: 'static + Fn(&Reference) -> Option<f64>>(
        &mut self,
        title: &str,
        map: F,
    );

    fn set_sample_additional<F: 'static + Fn(&Chromatography) -> Option<f64>>(
        &mut self,
        title: &str,
        map: F,
    );

    fn set_transpose(&mut self, enable: bool);

    fn build_existing_section<F: Fn(&Component) -> Option<f64>>(&mut self, title: &str, extract: F);
    fn build_expected_section<F: Fn(&Component) -> Option<f64>>(&mut self, title: &str, extract: F);

    fn build(self) -> T;
}

struct TableBuilderCsv<'a> {
    builder: String,
    transpose: bool,
    references: Rc<[Reference]>,
    reference_additional: Option<(String, Box<dyn Fn(&Reference) -> Option<f64>>)>,
    samples: &'a [Chromatography],
    sample_additional: Option<(String, Box<dyn Fn(&Chromatography) -> Option<f64>>)>,
}

impl<'a> TableBuilderCsv<'a> {
    fn new(references: Rc<[Reference]>, samples: &'a [Chromatography]) -> Self {
        Self {
            builder: String::new(),
            transpose: false,
            references,
            reference_additional: None,
            samples,
            sample_additional: None,
        }
    }

    fn format_maybe(&self, maybe_value: Option<f64>) -> String {
        match maybe_value {
            None => ",".to_string(),
            Some(value) => format!(",{:.3}", value),
        }
    }
}

impl<'a> TableBuilder<String> for TableBuilderCsv<'a> {
    fn set_reference_additional<F: 'static + Fn(&Reference) -> Option<f64>>(
        &mut self,
        title: &str,
        map: F,
    ) {
        self.reference_additional = Some((title.to_string(), Box::new(map)));
    }

    fn set_sample_additional<F: 'static + Fn(&Chromatography) -> Option<f64>>(
        &mut self,
        title: &str,
        map: F,
    ) {
        self.sample_additional = Some((title.to_string(), Box::new(map)));
    }

    fn set_transpose(&mut self, enable: bool) {
        self.transpose = enable;
    }

    fn build_existing_section<F: Fn(&Component) -> Option<f64>>(
        &mut self,
        title: &str,
        extract: F,
    ) {
        self.builder.push_str(&format!("[{}]", title));

        let filter = ComponentFilter {
            unknown: true,
            located: false,
            reference: false,
        };

        if self.transpose {
            self.builder.push_str("\nTitle");

            let mut max = 0;
            for sample in self.samples.iter() {
                if sample.get_components(&filter).len() > max {
                    max = sample.get_components(&filter).len();
                }
            }

            for i in 0..max {
                self.builder.push_str(&format!(",{}", i));
            }

            for sample in self.samples.iter() {
                self.builder.push_str(&format!("\n{}", sample.title));
                for component in sample.get_components(&filter) {
                    if let Some(entry) = extract(&component) {
                        self.builder.push_str(&format!(",{:.3}", entry));
                    } else {
                        self.builder.push_str(",");
                    }
                }
            }
        } else {
            self.builder.push_str("\nIndex");
            for sample in self.samples.iter() {
                self.builder.push_str(&format!(",{}", sample.title));
            }

            let iterators: Vec<Vec<Component>> = self
                .samples
                .iter()
                .map(|sample| sample.get_components(&filter))
                .collect();

            let mut index = 0;
            loop {
                let mut exhausted = true;
                self.builder.push_str(&format!("\n{}", index));
                for components in iterators.iter() {
                    if let Some(entry) = components.get(index).map_or(None, &extract) {
                        exhausted = false;
                        self.builder.push_str(&format!(",{:.3}", entry));
                    } else {
                        self.builder.push_str(",");
                    }
                }

                if exhausted {
                    break;
                }

                index += 1;
            }
        }

        self.builder.push_str("\n\n");
    }

    fn build_expected_section<F: Fn(&Component) -> Option<f64>>(
        &mut self,
        title: &str,
        extract: F,
    ) {
        let header = format!("[{}]\n", title);
        self.builder.push_str(&header);

        let sample_titles = self.samples.iter().fold(String::new(), |accum, sample| {
            format!("{},{}", accum, sample.title)
        });

        let lipid_titles = self
            .references
            .iter()
            .fold(String::new(), |accum, reference| {
                format!(
                    "{},{}",
                    accum,
                    reference.name.as_ref().map_or("[Unnamed]", |inner| &inner)
                )
            });

        if self.transpose {
            //? Rows = Samples, Columns = Lipids
            self.builder.push_str("Title");

            if let Some((title, _)) = &self.sample_additional {
                let header = format!(",{}", title);
                self.builder.push_str(&header);
            }

            self.builder.push_str(&lipid_titles);

            if let Some((title, extract_additional)) = &self.reference_additional {
                let entry = format!("\n{}", title);
                self.builder.push_str(&entry);
                for reference in self.references.iter() {
                    let maybe_entry = extract_additional(reference);
                    let entry = self.format_maybe(maybe_entry);
                    self.builder.push_str(&entry);
                }
            }

            for sample in self.samples.iter() {
                self.builder.push_str(&format!("\n{}", sample.title));

                if let Some((_, extract_additional)) = &self.sample_additional {
                    let maybe_entry = extract_additional(sample);
                    let entry = self.format_maybe(maybe_entry);
                    self.builder.push_str(&entry);
                }

                for lipid in sample
                    .get_components(&ComponentFilter::EXPECTED_ONLY)
                    .iter()
                {
                    let maybe_entry = extract(lipid);
                    let entry = self.format_maybe(maybe_entry);
                    self.builder.push_str(&entry);
                }
            }
        } else {
            self.builder.push_str("Lipid");

            if let Some((title, _)) = &self.reference_additional {
                let header = format!(",{}", title);
                self.builder.push_str(&header);
            }

            self.builder.push_str(&sample_titles);

            if let Some((title, extract_additional)) = &self.sample_additional {
                let entry = format!("\n{}", title);
                self.builder.push_str(&entry);
                for sample in self.samples.iter() {
                    let maybe_entry = extract_additional(sample);
                    let entry = self.format_maybe(maybe_entry);
                    self.builder.push_str(&entry);
                }
            }

            for (i, reference) in self.references.iter().enumerate() {
                let name = reference.name.as_ref().map_or("[Unnamed]", |inner| &inner);
                self.builder.push_str(&format!("\n{}", name));

                if let Some((_, extract_additional)) = &self.reference_additional {
                    let maybe_entry = extract_additional(reference);
                    let entry = self.format_maybe(maybe_entry);
                    self.builder.push_str(&entry);
                }

                for sample in self.samples.iter() {
                    let maybe_value = sample
                        .get_components(&ComponentFilter::EXPECTED_ONLY)
                        .get(i)
                        .map_or(None, &extract);
                    let entry = self.format_maybe(maybe_value);
                    self.builder.push_str(&entry);
                }
            }
        }

        self.reference_additional = None;
        self.sample_additional = None;

        self.builder.push_str("\n\n");
    }

    fn build(self) -> String {
        self.builder
    }
}

struct TableBuilderElement<'a> {
    builder: Vec<Element<'static, ()>>,
    transpose: bool,
    references: Rc<[Reference]>,
    reference_additional: Option<(String, Box<dyn Fn(&Reference) -> Option<f64>>)>,
    samples: &'a [Chromatography],
    sample_additional: Option<(String, Box<dyn Fn(&Chromatography) -> Option<f64>>)>,
}

impl<'a> TableBuilderElement<'a> {
    fn new(references: Rc<[Reference]>, samples: &'a [Chromatography]) -> Self {
        Self {
            builder: vec![],
            transpose: false,
            references,
            reference_additional: None,
            samples,
            sample_additional: None,
        }
    }

    fn format_maybe(&self, maybe_value: Option<f64>) -> String {
        match maybe_value {
            None => "".to_string(),
            Some(value) => format!("{:.3}", value),
        }
    }
}

impl<'a> TableBuilder<Element<'static, ()>> for TableBuilderElement<'a> {
    fn set_reference_additional<F: 'static + Fn(&Reference) -> Option<f64>>(
        &mut self,
        title: &str,
        map: F,
    ) {
        self.reference_additional = Some((title.to_string(), Box::new(map)));
    }

    fn set_sample_additional<F: 'static + Fn(&Chromatography) -> Option<f64>>(
        &mut self,
        title: &str,
        map: F,
    ) {
        self.sample_additional = Some((title.to_string(), Box::new(map)));
    }

    fn set_transpose(&mut self, enable: bool) {
        self.transpose = enable;
    }

    fn build_existing_section<F: Fn(&Component) -> Option<f64>>(
        &mut self,
        title: &str,
        extract: F,
    ) {
        const ENTRY_WIDTH: u16 = 70;

        let header = text(title.to_string()).into();
        self.builder.push(header);

        let filter = ComponentFilter {
            unknown: true,
            located: false,
            reference: false,
        };

        let mut table = column![];

        if self.transpose {
            let mut top = row![text("Title").width(ENTRY_WIDTH)];

            let mut max = 0;
            for sample in self.samples.iter() {
                if sample.get_components(&filter).len() > max {
                    max = sample.get_components(&filter).len();
                }
            }

            for i in 0..max {
                let index = text(format!("{}", i)).width(ENTRY_WIDTH);
                top = top.push(index);
            }

            table = table.push(top);

            for sample in self.samples.iter() {
                let mut line = row![];
                line = line.push(text(sample.title.clone()).width(ENTRY_WIDTH));
                for component in sample.get_components(&filter) {
                    if let Some(entry) = extract(&component) {
                        line = line.push(text(format!("{:.3}", entry)).width(ENTRY_WIDTH));
                    } else {
                        line = line.push(text("").width(ENTRY_WIDTH));
                    }
                }

                table = table.push(line);
            }
        } else {
            let mut top = row![text("Index").width(ENTRY_WIDTH)];

            for sample in self.samples.iter() {
                top = top.push(text(sample.title.clone()).width(ENTRY_WIDTH));
            }

            table = table.push(top);

            let iterators: Vec<Vec<Component>> = self
                .samples
                .iter()
                .map(|sample| sample.get_components(&filter))
                .collect();

            let mut index = 0;
            loop {
                let mut exhausted = true;
                let mut line: widget::Row<'_, _, _, _> = row![];
                line = line.push(text(index).width(ENTRY_WIDTH));
                for components in iterators.iter() {
                    if let Some(entry) = components.get(index).map_or(None, &extract) {
                        exhausted = false;
                        line = line.push(text(format!("{:.3}", entry)).width(ENTRY_WIDTH));
                    } else {
                        line = line.push(text("").width(ENTRY_WIDTH));
                    }
                }

                table = table.push(line);

                if exhausted {
                    break;
                }

                index += 1;
            }
        }

        self.builder.push(table.into());
    }

    fn build_expected_section<F: Fn(&Component) -> Option<f64>>(
        &mut self,
        title: &str,
        extract: F,
    ) {
        const ENTRY_WIDTH: u16 = 70;

        let header = text(title.to_string()).into();
        self.builder.push(header);

        let mut table: Column<'static, ()> = column![];

        let sample_to_text = |sample: &Chromatography| {
            let name = sample.title.clone();
            text(name).width(ENTRY_WIDTH).into()
        };

        let sample_titles: Vec<Element<'static, ()>> =
            self.samples.iter().map(sample_to_text).collect();

        let lipid_to_text = |reference: &Reference| {
            let name = reference.name.clone().unwrap_or("[Unnamed]".to_string());
            text(name).width(ENTRY_WIDTH).into()
        };

        let lipid_titles: Vec<Element<'static, ()>> =
            self.references.iter().map(lipid_to_text).collect();

        if self.transpose {
            //? Rows = Samples, Columns = Lipids
            let mut headers = row![text("Title").width(ENTRY_WIDTH)].spacing(5);

            if let Some((title, _)) = &self.sample_additional {
                headers = headers.push(text(title.clone()).width(ENTRY_WIDTH));
            }

            headers = headers.extend(lipid_titles);
            table = table.push(headers);

            if let Some((title, extract_additional)) = &self.reference_additional {
                let mut additional_row = row![text(title.clone()).width(ENTRY_WIDTH)].spacing(5);
                for reference in self.references.iter() {
                    let maybe_entry = extract_additional(reference);
                    let entry = self.format_maybe(maybe_entry);
                    additional_row = additional_row.push(text(entry).width(ENTRY_WIDTH));
                }

                table = table.push(additional_row);
            }

            for sample in self.samples.iter() {
                let mut standard_row =
                    row![text(sample.title.clone()).width(ENTRY_WIDTH)].spacing(5);

                if let Some((_, extract_additional)) = &self.sample_additional {
                    let maybe_entry = extract_additional(sample);
                    let entry = self.format_maybe(maybe_entry);
                    standard_row = standard_row.push(text(entry).width(ENTRY_WIDTH));
                }

                for lipid in sample
                    .get_components(&ComponentFilter::EXPECTED_ONLY)
                    .iter()
                {
                    let maybe_entry = extract(lipid);
                    let entry = self.format_maybe(maybe_entry);
                    standard_row = standard_row.push(text(entry).width(ENTRY_WIDTH));
                }

                table = table.push(standard_row);
            }
        } else {
            let mut headers = row![text("Lipid").width(ENTRY_WIDTH)].spacing(5);

            if let Some((title, _)) = &self.reference_additional {
                headers = headers.push(text(title.clone()).width(ENTRY_WIDTH));
            }

            headers = headers.extend(sample_titles);
            table = table.push(headers);

            if let Some((title, extract_additional)) = &self.sample_additional {
                let mut additional_row = row![text(title.clone()).width(ENTRY_WIDTH)].spacing(5);
                for sample in self.samples.iter() {
                    let maybe_entry = extract_additional(sample);
                    let entry = self.format_maybe(maybe_entry);
                    additional_row = additional_row.push(text(entry).width(ENTRY_WIDTH));
                }

                table = table.push(additional_row);
            }

            for (i, reference) in self.references.iter().enumerate() {
                let name = reference.name.clone().unwrap_or("[Unnamed]".to_string());
                let mut standard_row = row![text(name).width(ENTRY_WIDTH)].spacing(5);

                if let Some((_, extract_additional)) = &self.reference_additional {
                    let maybe_entry = extract_additional(reference);
                    let entry = self.format_maybe(maybe_entry);
                    standard_row = standard_row.push(text(entry).width(ENTRY_WIDTH));
                }

                for sample in self.samples.iter() {
                    let maybe_value = sample
                        .get_components(&ComponentFilter::EXPECTED_ONLY)
                        .get(i)
                        .map_or(None, &extract);
                    let entry = self.format_maybe(maybe_value);
                    standard_row = standard_row.push(text(entry).width(ENTRY_WIDTH));
                }

                table = table.push(standard_row);
            }
        }

        self.builder.push(table.into());

        let spacer = Space::new(Length::Fill, 100);
        self.builder.push(spacer.into());
    }

    fn build(self) -> Element<'static, ()> {
        Column::from_vec(self.builder)
            .align_x(Horizontal::Center)
            .into()
    }
}
