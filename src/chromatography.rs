use std::ffi::OsString;
use std::fs;
use std::iter::Iterator;
use std::ops::Range;
use std::path::Path;
use std::rc::Rc;

use iced::color;
use iced::widget::{container, row, scrollable, text};
use iced::{Element, Point, widget::column};

use crate::component::{Component, Peak};
use crate::reference::Reference;
use crate::spline::Spline;
use crate::vector::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SampleType {
    #[default]
    Data,
    Blank,
    Dex,
    Standard,
}

pub struct ComponentFilter {
    pub unknown: bool,
    pub located: bool,
    pub reference: bool,
}

impl ComponentFilter {
    pub const EXISTING_ONLY: ComponentFilter = ComponentFilter {
        unknown: true,
        located: true,
        reference: false,
    };

    pub const EXPECTED_ONLY: ComponentFilter = ComponentFilter {
        unknown: false,
        located: true,
        reference: true,
    };
}

//TODO: parametrise constants
static MEAN_FILTER_RANGE: usize = 5;
static DEX_RANGE: Range<f64> = 0.0..38.7;

#[derive(Clone, Debug, Default)]
pub struct Chromatography {
    // Transformations of the data
    raw_data: Vec<Point2D>,
    cleaned_data: Vec<Point2D>,
    first_derivative: Vec<Point2D>,
    second_derivative: Vec<Point2D>,
    pub baseline: Vec<Point2D>,
    pub total_area: f64,

    // Derived components
    existing_components: Vec<Peak>,
    qualified_components: Vec<Component>,

    // Chromatography configuration
    sample_type: SampleType,
    data_range: Option<Range<f64>>,
    include_unknowns: bool,
    height_requirement: f64,
    inflection_requirement: f64,
    retention_time_tolerance: f64,
    glucose_unit_tolerance: f64,

    // External references
    lipid_references: Rc<[Reference]>,
    glucose_transformer: Option<Spline>,

    // Display configuration + state
    //TODO: how can we not put data that is only for rendering here?
    pub title: String,
    pub file_name: OsString,
    pub global_zoom: Point<f64>,
}

impl Chromatography {
    pub fn from_file<T: AsRef<Path>>(path: &T) -> Option<Self> {
        let file = {
            match fs::read_to_string(path) {
                Ok(content) => content,
                Err(_) => return None,
            }
        };

        let mut empty = Chromatography::default();

        empty.file_name = path.as_ref().file_name().unwrap().to_os_string();

        let mut splitter = String::new();
        if file.contains('\r') {
            splitter += "\r";
        }
        if file.contains('\n') {
            splitter += "\n";
        }

        empty.title = {
            let mut name = empty.file_name.clone().into_string().unwrap();

            for line in file.split(&splitter) {
                let mut pair = line.split("\t");
                if let Some(key) = pair.next() {
                    if key == "\"SampleName\"" {
                        name = pair.next().unwrap().to_string();
                    }
                }
            }

            name
        };

        empty.raw_data = file
            .split(&splitter)
            .filter_map(|line| {
                let mut data = if line.contains("\t") {
                    line.split("\t")
                } else {
                    line.split(",")
                };

                let x: f64 = match data.next() {
                    Some(string) => match string.parse::<f64>() {
                        Ok(value) => value,
                        Err(_) => return None,
                    },
                    None => return None,
                };

                let y: f64 = match data.next() {
                    Some(string) => match string.parse::<f64>() {
                        Ok(value) => value,
                        Err(_) => return None,
                    },
                    None => return None,
                };

                Some(Point2D::new(x, y))
            })
            .collect();

        empty.cleaned_data = Self::mean_filter(&empty.raw_data, MEAN_FILTER_RANGE);
        empty.first_derivative = Self::calculate_derivative(&empty.cleaned_data);
        empty.second_derivative = Self::calculate_derivative(&empty.first_derivative);
        empty.baseline = empty.calculate_baseline();
        empty.total_area = empty.calculate_area();
        empty.existing_components = empty.calculate_components();
        empty.qualified_components = empty.identify_components();

        Some(empty)
    }

    pub fn get_data(&self) -> Vec<Point2D> {
        self.cleaned_data.clone()
    }

    pub fn get_data_range(&self) -> Range<f64> {
        if let Some(range) = &self.data_range {
            range.clone()
        } else {
            let default = &Point2D::default();
            let end = self.raw_data.last().unwrap_or(default);
            0.0..end.x()
        }
    }

    pub fn set_data_range(&mut self, value: &Range<f64>) -> &mut Self {
        self.data_range = Some(value.clone());

        if self.sample_type == SampleType::Dex {
            self.cleaned_data = Self::mean_filter(&self.raw_data, MEAN_FILTER_RANGE)
                .iter()
                .filter(|point| DEX_RANGE.contains(&point.x()))
                .cloned()
                .collect();
        } else {
            self.cleaned_data = Self::mean_filter(&self.raw_data, MEAN_FILTER_RANGE)
                .iter()
                .filter(|point| value.contains(&point.x()))
                .cloned()
                .collect();
        }

        self.first_derivative = Self::calculate_derivative(&self.cleaned_data);
        self.second_derivative = Self::calculate_derivative(&self.first_derivative);
        self.baseline = self.calculate_baseline();
        self.total_area = self.calculate_area();
        self.existing_components = self.calculate_components();
        self.qualified_components = self.identify_components();

        self
    }

    pub fn get_highest_point(&self) -> f64 {
        let mut highest = 0.0;
        for point in &self.cleaned_data {
            if point.y() > highest {
                highest = point.y();
            }
        }

        highest
    }

    pub fn get_unqualified_components(&self) -> Vec<Peak> {
        self.existing_components.clone()
    }

    pub fn get_components(&self, filter: &ComponentFilter) -> Vec<Component> {
        self.qualified_components
            .iter()
            .cloned()
            .filter(|component| match component {
                Component::Unknown(_) => filter.unknown,
                Component::Located(_, _) => filter.located,
                Component::Reference(_) => filter.reference,
            })
            .collect()
    }

    pub fn set_lipid_references(&mut self, value: Rc<[Reference]>) -> &mut Self {
        self.lipid_references = value;
        self.qualified_components = self.identify_components();

        self
    }

    pub fn set_include_unknowns(&mut self, show: &bool) -> &mut Self {
        self.include_unknowns = *show;
        self.existing_components = self.calculate_components();
        self.qualified_components = self.identify_components();

        self
    }

    pub fn set_height_requirement(&mut self, value: &f64) -> &mut Self {
        self.height_requirement = *value;
        self.existing_components = self.calculate_components();
        self.qualified_components = self.identify_components();

        self
    }

    pub fn set_inflection_requirement(&mut self, value: &f64) -> &mut Self {
        self.inflection_requirement = *value;
        self.existing_components = self.calculate_components();
        self.qualified_components = self.identify_components();

        self
    }

    pub fn set_retention_time_tolerance(&mut self, value: &f64) -> &mut Self {
        self.retention_time_tolerance = *value;
        self.qualified_components = self.identify_components();

        self
    }

    pub fn set_glucose_unit_tolerance(&mut self, value: &f64) -> &mut Self {
        self.glucose_unit_tolerance = *value;
        self.qualified_components = self.identify_components();

        self
    }

    pub fn get_glucose_transformer(&self) -> Option<Spline> {
        let mut peaks = self.existing_components.clone();
        peaks.reverse();

        let mut height = 0.0;
        let mut intersections: Vec<Point2D> = vec![];
        for peak in &peaks {
            if peak.height > height {
                height = peak.height;
                intersections.push(peak.retention_point.clone());
            }
        }

        intersections.reverse();
        let points: Vec<Point2D> = intersections
            .iter()
            .enumerate()
            .map(|(i, point)| Point2D::new(point.x(), (i + 1) as f64))
            .collect();

        Spline::new(&points)
    }

    pub fn set_glucose_transformer(&mut self, transformer: &Option<Spline>) -> &mut Self {
        self.glucose_transformer = transformer.clone();
        self.existing_components = self.calculate_components();
        self.qualified_components = self.identify_components();

        self
    }

    pub fn get_sample_type(&self) -> SampleType {
        self.sample_type
    }

    pub fn set_sample_type(&mut self, value: &SampleType) -> &mut Self {
        self.sample_type = value.clone();

        if *value == SampleType::Dex {
            self.set_data_range(&DEX_RANGE);
        }

        self
    }

    fn mean_filter(data: &[Point2D], smoothing: usize) -> Vec<Point2D> {
        let mut smoothed = Vec::with_capacity(data.len());

        for i in smoothing..data.len() - smoothing {
            let mut total = 0.0;
            let start = i - smoothing;
            let end = i + smoothing;

            for offset in start..=end {
                total += data[offset].y();
            }

            let point = Point2D::new(data[i].x(), total / (1 + end - start) as f64);
            smoothed.push(point);
        }

        smoothed
    }

    fn calculate_derivative(graph: &[Point2D]) -> Vec<Point2D> {
        if graph.len() < 2 {
            return vec![];
        }

        let mut derivative = Vec::with_capacity(graph.len());

        for i in 1..graph.len() {
            let prev = &graph[i - 1];
            let next = &graph[i];

            let point = Point2D::new(prev.x(), prev.gradient(next));
            derivative.push(point);
        }

        derivative
    }

    fn calculate_baseline(&self) -> Vec<Point2D> {
        let data = &self.cleaned_data;

        let mut origin = &data[0];
        let mut orgin_index = 0;

        let mut next = &data[1];
        let mut next_index = 1;

        let mut baseline = vec![];

        while next_index + 1 < data.len() {
            let mut best_gradient = f64::INFINITY;

            for i in orgin_index..data.len() {
                let point = &data[i];

                let gradient = origin.gradient(point);
                if gradient < best_gradient {
                    next = point;
                    next_index = i;
                    best_gradient = gradient;
                }
            }

            for index in orgin_index..next_index {
                let time = data[index].x();
                let x = time - data[orgin_index].x();
                let height = best_gradient * x + origin.y();
                baseline.push(Point2D::new(time, height));
            }

            origin = &next;
            orgin_index = next_index;
        }

        baseline.push(*next);
        baseline
    }

    fn calculate_area(&self) -> f64 {
        let mut area = 0.0;
        for i in 1..self.cleaned_data.len() {
            let prev = self.cleaned_data[i - 1];
            let next = self.cleaned_data[i];

            let width = next.x() - prev.x();
            let a = prev.y() - self.baseline[i - 1].y();
            let b = next.y() - self.baseline[i].y();
            area += 0.5 * width * (a + b);
        }

        return area;
    }

    fn calculate_components(&self) -> Vec<Peak> {
        let pivot = 4;
        let mut result = vec![];

        let mut peak = Peak::default();
        peak.start = self.cleaned_data[pivot].clone();

        let mut found_maximum = false;
        let mut prev_min = self.cleaned_data[pivot].y() - self.baseline[pivot].y();

        for index in pivot..self.cleaned_data.len() {
            let prev = &self.cleaned_data[index - 1];
            let next = &self.cleaned_data[index];
            let height = prev.y() - self.baseline[index - 1].y();

            let area = {
                let h = next.x() - prev.x();
                let a = prev.y() - self.baseline[index - 1].y();
                let b = next.y() - self.baseline[index].y();
                h * (a + b) / 2.0
            };

            peak.area += area;

            let prev_drv = &self.first_derivative[index - 2];
            let next_drv = &self.first_derivative[index - 1];

            // Minimum
            if prev_drv.y() <= 0.0 && next_drv.y() >= 0.0 {
                // Real peak
                if found_maximum {
                    found_maximum = false;
                    prev_min = height;

                    peak.end = prev.clone();

                    result.push(peak);
                    peak = Peak::default();
                    peak.start = prev.clone();
                    // Lower minimum but without a peak, merge with prev peak
                } else if height < prev_min {
                    prev_min = height;

                    if let Some(prev_peak) = result.last_mut() {
                        prev_peak.end = prev.clone();
                        prev_peak.area += peak.area;
                    }

                    peak = Peak::default();
                    peak.start = prev.clone();
                }
            } else if prev_drv.y() >= 0.0 && next_drv.y() <= 0.0 {
                // Maximum
                peak.height = prev.y() - self.baseline[index - 1].y();
                if peak.height > self.height_requirement {
                    found_maximum = true;
                }

                if prev.y() > next.y() {
                    peak.retention_point = prev.clone();
                } else {
                    peak.retention_point = next.clone();
                }
            }

            let prev_drv2 = &self.second_derivative[index - 3];
            let next_drv2 = &self.second_derivative[index - 2];

            let difference = f64::abs(prev_drv2.y() - next_drv2.y());

            if difference < self.inflection_requirement || height < self.height_requirement {
                continue;
            }

            let rising_zero = prev_drv2.y() <= 0.0 && next_drv2.y() >= 0.0;

            if rising_zero && prev_drv.y() >= 0.0 {
                peak.height = next.y() - self.baseline[index].y();
                peak.retention_point = next.clone();
                peak.end = next.clone();
                result.push(peak);

                peak = Peak::default();
                peak.start = next.clone();
            }

            let falling_zero = prev_drv2.y() >= 0.0 && next_drv2.y() <= 0.0;

            if falling_zero && prev_drv.y() <= 0.0 {
                peak.end = next.clone();
                result.push(peak);

                peak = Peak::default();
                peak.start = next.clone();
                peak.retention_point = next.clone();
                peak.height = next.y() - self.baseline[index].y();
            }
        }

        result
    }

    fn identify_components(&self) -> Vec<Component> {
        // We need to create 3 lists here
        // * Only components present in the reference (Located + Reference)
        // * Only components present in the sample but with metadata attached (Located + Unknown)
        // * All components we could possibly include (Unknown + Located + Reference)

        if self.existing_components.is_empty() && self.lipid_references.is_empty() {
            return vec![];
        }

        if self.lipid_references.is_empty() {
            return self
                .existing_components
                .iter()
                .map(|peak| Component::Unknown(peak.clone()))
                .collect();
        }

        if self.existing_components.is_empty() {
            return self
                .lipid_references
                .iter()
                .map(|reference| Component::Reference(reference.clone()))
                .collect();
        }

        let tolerance = match self.glucose_transformer {
            Some(_) => self.glucose_unit_tolerance,
            None => self.retention_time_tolerance,
        };

        let mut available_references: Vec<Option<&Reference>> = self
            .lipid_references
            .iter()
            .map(|reference| Some(reference))
            .collect();

        let mut available_components: Vec<Option<&Peak>> = self
            .existing_components
            .iter()
            .map(|component| Some(component))
            .collect();

        //? This is O(n^3)
        let mut located_components = vec![];
        loop {
            let mut shortest_distance = None;
            let mut best_reference = None;
            let mut best_component = None;

            for (i, maybe_reference) in available_references.iter().enumerate() {
                let maybe_location = maybe_reference.map_or(None, |reference| {
                    reference.get_expected_location(self.glucose_transformer.as_ref())
                });
                let expected_location = match maybe_location {
                    Some(location) => location,
                    None => continue,
                };

                for (j, maybe_component) in available_components.iter().enumerate() {
                    let maybe_location = maybe_component.map_or(None, |component| {
                        component.get_retention_location(self.glucose_transformer.as_ref())
                    });
                    let component_location = match maybe_location {
                        Some(location) => location,
                        None => continue,
                    };

                    let distance = f64::abs(component_location - expected_location);
                    if let Some(best_distance) = shortest_distance {
                        if distance < best_distance {
                            shortest_distance = Some(distance);
                            best_reference = Some(i);
                            best_component = Some(j);
                        }
                    } else {
                        shortest_distance = Some(distance);
                        best_reference = Some(i);
                        best_component = Some(j);
                    }
                }
            }

            match (shortest_distance, best_reference, best_component) {
                (Some(distance), Some(reference), Some(component)) => {
                    if distance > tolerance {
                        break;
                    }

                    let borrow_ref = available_references[reference].unwrap();
                    let borrow_component = available_components[component].unwrap();
                    located_components.push((borrow_component, borrow_ref));

                    available_references[reference] = None;
                    available_components[component] = None;
                }
                _ => {
                    break;
                }
            }
        }

        located_components.sort_by(|left, right| {
            left.0
                .retention_point
                .x()
                .total_cmp(&right.0.retention_point.x())
        });

        // Time to iterate across 3 vectors at once
        // It won't be pretty.
        let mut complete_components = vec![];

        let mut unknown_components = self
            .existing_components
            .iter()
            .filter_map(|peak| {
                if let Some(location) =
                    peak.get_retention_location(self.glucose_transformer.as_ref())
                {
                    Some((peak, location))
                } else {
                    None
                }
            })
            .peekable();

        let mut located_components = located_components
            .iter()
            .filter_map(|(peak, reference)| {
                if let Some(location) =
                    peak.get_retention_location(self.glucose_transformer.as_ref())
                {
                    Some((*peak, *reference, location))
                } else {
                    None
                }
            })
            .peekable();

        let mut reference_components = self
            .lipid_references
            .iter()
            .filter_map(|reference| {
                if let Some(location) =
                    reference.get_expected_location(self.glucose_transformer.as_ref())
                {
                    Some((reference, location))
                } else {
                    None
                }
            })
            .peekable();

        loop {
            match (
                unknown_components.peek(),
                located_components.peek(),
                reference_components.peek(),
            ) {
                (None, None, None) => break,
                (None, None, Some((reference, _))) => {
                    complete_components.push(Component::Reference((*reference).clone()));
                    reference_components.next();
                }
                (None, Some((located_peak, located_reference, _)), None) => {
                    complete_components.push(Component::Located(
                        (*located_peak).clone(),
                        (*located_reference).clone(),
                    ));
                    located_components.next();
                }
                (
                    None,
                    Some((located_peak, located_reference, located_location)),
                    Some((reference, expected_location)),
                ) => {
                    if located_reference == reference {
                        complete_components.push(Component::Located(
                            (*located_peak).clone(),
                            (*located_reference).clone(),
                        ));
                        located_components.next();
                        reference_components.next();
                        continue;
                    }

                    if located_location < expected_location {
                        complete_components.push(Component::Located(
                            (*located_peak).clone(),
                            (*located_reference).clone(),
                        ));
                        located_components.next();
                    } else {
                        complete_components.push(Component::Reference((*reference).clone()));
                        reference_components.next();
                    }
                }
                (Some((unknown, _)), None, None) => {
                    complete_components.push(Component::Unknown((*unknown).clone()));
                    unknown_components.next();
                }
                (Some((unknown, unknown_location)), None, Some((reference, expected_location))) => {
                    if unknown_location < expected_location {
                        complete_components.push(Component::Unknown((*unknown).clone()));
                        unknown_components.next();
                    } else {
                        complete_components.push(Component::Reference((*reference).clone()));
                        reference_components.next();
                    }
                }
                (
                    Some((unknown, unknown_location)),
                    Some((located_peak, located_reference, located_location)),
                    None,
                ) => {
                    if located_peak == unknown {
                        complete_components.push(Component::Located(
                            (*located_peak).clone(),
                            (*located_reference).clone(),
                        ));
                        located_components.next();
                        unknown_components.next();
                        continue;
                    }

                    if unknown_location < located_location {
                        complete_components.push(Component::Unknown((*unknown).clone()));
                        unknown_components.next();
                    } else {
                        complete_components.push(Component::Located(
                            (*located_peak).clone(),
                            (*located_reference).clone(),
                        ));
                        located_components.next();
                    }
                }
                (
                    Some((unknown, unknown_location)),
                    Some((located_peak, located_reference, located_location)),
                    Some((reference, expected_location)),
                ) => {
                    if located_peak == unknown && located_reference == reference {
                        complete_components.push(Component::Located(
                            (*located_peak).clone(),
                            (*located_reference).clone(),
                        ));
                        located_components.next();
                        unknown_components.next();
                        reference_components.next();
                        continue;
                    }

                    let merge_peaks = located_peak == unknown;
                    let merge_references = located_reference == reference;

                    if located_location <= unknown_location && located_location <= expected_location
                    {
                        complete_components.push(Component::Located(
                            (*located_peak).clone(),
                            (*located_reference).clone(),
                        ));
                        located_components.next();

                        if merge_peaks {
                            unknown_components.next();
                        }

                        if merge_references {
                            reference_components.next();
                        }

                        continue;
                    }

                    if unknown_location < expected_location {
                        complete_components.push(Component::Unknown((*unknown).clone()));
                        unknown_components.next();
                    } else {
                        complete_components.push(Component::Reference((*reference).clone()));
                        reference_components.next();
                    }
                }
            }
        }

        complete_components
    }

    pub fn set_global_zoom(&mut self, zoom: &Point<f64>) -> &mut Self {
        self.global_zoom = zoom.clone();

        self
    }

    pub fn into_table_element<'b>(&'b self, concentration_multiplier: f64) -> Element<'b, ()> {
        let mut table = column![];
        let title = text(format!("Total Area - {}", self.total_area))
            .width(950)
            .center();

        let mut gray = container::Style::default();
        gray = gray.background(color!(0xaaaaaa));

        let lipid_label = text("Lipid").center().width(200);
        let retention_label = text("RT (m) (found/expected)").center().width(200);
        let glucose_unit_label = text("GU (found/expected)").center().width(200);
        let area_label = text("Area").center().width(150);
        let concentration_label = text("Concentration (nmol/ml)").center().width(200);

        let header = row![
            text("|"),
            container(lipid_label).style(move |_| gray),
            text("|"),
            container(retention_label).style(move |_| gray),
            text("|"),
            container(glucose_unit_label).style(move |_| gray),
            text("|"),
            container(area_label).style(move |_| gray),
            text("|"),
            container(concentration_label).style(move |_| gray),
            text("|"),
        ]
        .spacing(20);

        let spacer_string = "-".repeat(215);

        table = table.push(title);
        table = table.push(text(spacer_string.clone()));
        table = table.push(header);

        for component in self.qualified_components.iter() {
            let name = match component {
                Component::Unknown(_) => "[Unknown]".to_string(),
                Component::Located(_, reference) => {
                    reference.name.clone().unwrap_or("[Unnamed]".to_string())
                }
                Component::Reference(reference) => {
                    reference.name.clone().unwrap_or("[Unnamed]".to_string())
                }
            };

            let retention_time = {
                let mut builder = String::new();

                if let Some(experimental) = component.get_experimental_rt() {
                    builder.push_str(&format!("{:.2}", experimental));
                } else {
                    builder.push_str("None");
                }

                if let Some(expected) = component.get_expected_rt() {
                    builder.push_str(&format!("/{:.2}", expected));
                }

                builder
            };

            let glucose_units = "H".to_string();

            let area = {
                let mut builder = String::new();

                if let Some(area) = component.get_area() {
                    builder.push_str(&format!("{:.2}", area));
                } else {
                    builder.push_str("None");
                }

                builder
            };

            let concentration = {
                let mut builder = String::new();

                if let Some(area) = component.get_area() {
                    builder.push_str(&format!("{:.2}", area * concentration_multiplier));
                } else {
                    builder.push_str("None");
                }

                builder
            };

            let content = row![
                text("|"),
                text(name).center().width(200),
                text("|"),
                text(retention_time).center().width(200),
                text("|"),
                text(glucose_units).center().width(200),
                text("|"),
                text(area).center().width(150),
                text("|"),
                text(concentration).center().width(200),
                text("|"),
            ]
            .spacing(20);

            table = table.push(text(spacer_string.clone()));
            table = table.push(content);
        }

        table = table.push(text(spacer_string.clone()));
        scrollable(table).height(200).into()
    }
}
