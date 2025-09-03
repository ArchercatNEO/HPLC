use std::ffi::OsString;
use std::fs;
use std::iter::Iterator;
use std::ops::Range;
use std::path::Path;
use std::rc::Rc;

use iced::color;
use iced::widget::{container, row, scrollable, text};
use iced::{Element, Point, widget::column};

use crate::peak::{Peak, PeakType};
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

#[derive(Clone, Debug, Default)]
pub struct Chromatography {
    // Transformations of the data
    raw_data: Vec<Point2D>,
    cleaned_data: Vec<Point2D>,
    first_derivative: Vec<Point2D>,
    second_derivative: Vec<Point2D>,
    pub baseline: Vec<Point2D>,
    pub peaks: Vec<Peak>,
    pub lipids: Vec<Peak>,
    pub total_area: f64,

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

        empty.title = {
            let mut name = empty.file_name.clone().into_string().unwrap();

            for line in file.split('\r') {
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
            .split('\r')
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

        // TODO: parametrice smoothing factor
        empty.cleaned_data = Self::mean_filter(&empty.raw_data, 5);
        empty.first_derivative = Self::calculate_derivative(&empty.cleaned_data);
        empty.second_derivative = Self::calculate_derivative(&empty.first_derivative);
        empty.baseline = empty.calculate_baseline(None);
        empty.total_area = empty.calculate_area(None);
        empty.peaks = empty.calculate_peaks(None);
        empty.lipids = empty.label_peaks();

        Some(empty)
    }

    pub fn get_data(&self) -> Vec<Point2D> {
        match (self.sample_type, &self.data_range) {
            (SampleType::Dex, _) => self
                .cleaned_data
                .iter()
                .filter(|point| (0.0..38.7).contains(&point.x()))
                .cloned()
                .collect(),
            (_, Some(range)) => self
                .cleaned_data
                .iter()
                .filter(|point| range.contains(&point.x()))
                .cloned()
                .collect(),
            (_, None) => self.cleaned_data.clone(),
        }
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
        if self.sample_type != SampleType::Dex {
            self.baseline = self.calculate_baseline(self.data_range.as_ref());
            self.total_area = self.calculate_area(self.data_range.as_ref());
            self.peaks = self.calculate_peaks(self.data_range.as_ref());
            self.lipids = self.label_peaks();
        }

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

    pub fn set_lipid_references(&mut self, value: Rc<[Reference]>) -> &mut Self {
        self.lipid_references = value;
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_include_unknowns(&mut self, show: &bool) -> &mut Self {
        self.include_unknowns = *show;
        self.peaks = self.calculate_peaks(self.data_range.as_ref());
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_height_requirement(&mut self, value: &f64) -> &mut Self {
        self.height_requirement = *value;
        self.peaks = self.calculate_peaks(self.data_range.as_ref());
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_inflection_requirement(&mut self, value: &f64) -> &mut Self {
        self.inflection_requirement = *value;
        self.peaks = self.calculate_peaks(self.data_range.as_ref());
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_retention_time_tolerance(&mut self, value: &f64) -> &mut Self {
        self.retention_time_tolerance = *value;
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_glucose_unit_tolerance(&mut self, value: &f64) -> &mut Self {
        self.glucose_unit_tolerance = *value;
        self.lipids = self.label_peaks();

        self
    }

    pub fn get_glucose_transformer(&self) -> Option<Spline> {
        let mut peaks = self.peaks.clone();
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
        self.peaks = self.calculate_peaks(self.data_range.as_ref());
        self.lipids = self.label_peaks();

        self
    }

    pub fn get_sample_type(&self) -> SampleType {
        self.sample_type
    }

    pub fn set_sample_type(&mut self, value: &SampleType) -> &mut Self {
        self.sample_type = value.clone();

        if *value == SampleType::Dex {
            let dex_range = Some(0.0..38.7);
            self.baseline = self.calculate_baseline(dex_range.as_ref());
            self.total_area = self.calculate_area(dex_range.as_ref());
            self.peaks = self.calculate_peaks(dex_range.as_ref());
            self.lipids = self.label_peaks();
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

    fn calculate_baseline(&self, maybe_range: Option<&Range<f64>>) -> Vec<Point2D> {
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

                if let Some(range) = maybe_range {
                    if point.x() > range.end {
                        break;
                    }
                }
            }

            for index in orgin_index..next_index {
                let time = data[index].x();
                let x = time - data[orgin_index].x();
                let height = best_gradient * x + origin.y();
                baseline.push(Point2D::new(time, height));
            }

            if let Some(range) = maybe_range {
                if next.x() > range.end {
                    let time = data[next_index].x();
                    let x = time - data[orgin_index].x();
                    let height = best_gradient * x + origin.y();
                    baseline.push(Point2D::new(time, height));
                    break;
                }
            }

            origin = &next;
            orgin_index = next_index;
        }

        baseline.push(*next);
        baseline
    }

    fn calculate_area(&self, maybe_range: Option<&Range<f64>>) -> f64 {
        let mut area = 0.0;
        for i in 1..self.cleaned_data.len() {
            let prev = self.cleaned_data[i - 1];
            let next = self.cleaned_data[i];

            if let Some(range) = maybe_range {
                if prev.x() < range.start {
                    continue;
                }

                if next.x() > range.end {
                    break;
                }
            }

            let width = next.x() - prev.x();
            let a = prev.y() - self.baseline[i - 1].y();
            let b = next.y() - self.baseline[i].y();
            area += 0.5 * width * (a + b);
        }

        return area;
    }

    fn calculate_peaks(&self, maybe_range: Option<&Range<f64>>) -> Vec<Peak> {
        let mut pivot = 4;
        if let Some(range) = maybe_range {
            while self.cleaned_data[pivot].x() < range.start {
                pivot += 1;
            }
        }

        let mut result = vec![];

        let mut peak = Peak::default();
        peak.start = self.cleaned_data[pivot].clone();

        let mut found_maximum = false;
        let mut prev_min = self.cleaned_data[pivot].y() - self.baseline[pivot].y();

        for index in pivot..self.cleaned_data.len() {
            let prev = &self.cleaned_data[index - 1];
            let next = &self.cleaned_data[index];

            if let Some(range) = maybe_range {
                if next.x() > range.end {
                    break;
                }
            }

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
                peak.height = prev.y() - peak.start.y();
                if peak.height > self.height_requirement {
                    found_maximum = true;
                }

                if prev.y() > next.y() {
                    peak.retention_point = prev.clone();
                } else {
                    peak.retention_point = next.clone();
                }
            } else {
            }

            let prev_drv2 = &self.second_derivative[index - 3];
            let next_drv2 = &self.second_derivative[index - 2];

            let difference = f64::abs(prev_drv2.y() - next_drv2.y());

            if difference < self.inflection_requirement || height < self.height_requirement {
                continue;
            }

            let rising_zero = prev_drv2.y() <= 0.0 && next_drv2.y() >= 0.0;

            if rising_zero && prev_drv.y() >= 0.0 {
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
            }
        }

        if let Some(spline) = &self.glucose_transformer {
            for peak in result.iter_mut() {
                peak.gu = spline.evaluate(peak.retention_point.x());
            }
        }

        result
    }

    fn label_peaks(&mut self) -> Vec<Peak> {
        for peak in self.peaks.iter_mut() {
            peak.peak_type = PeakType::Unknown;
        }

        let tolerance = match self.glucose_transformer {
            Some(_) => self.glucose_unit_tolerance,
            None => self.retention_time_tolerance,
        };

        let mut matrix: Vec<Vec<Option<f64>>> = Vec::with_capacity(self.lipid_references.len());
        for reference in self.lipid_references.iter() {
            let time = match &self.glucose_transformer {
                None => match &reference.retention_time {
                    Some(rt) => *rt,
                    None => {
                        println!("Broken lipid reference {:?}", reference.name);
                        matrix.push(vec![None; self.peaks.len()]);
                        continue;
                    }
                },
                Some(spline) => match &reference.glucose_units {
                    Some(gu) => *gu,
                    None => match &reference.retention_time {
                        Some(rt) => {
                            let maybe_gu = spline.evaluate(*rt);
                            match maybe_gu {
                                Some(gu) => gu,
                                None => {
                                    println!("Lipid {:?} out of range", reference.name);
                                    matrix.push(vec![None; self.peaks.len()]);
                                    continue;
                                }
                            }
                        }
                        None => {
                            println!("Broken lipid reference {:?}", reference.name);
                            matrix.push(vec![None; self.peaks.len()]);
                            continue;
                        }
                    },
                },
            };

            let mut distances: Vec<Option<f64>> = Vec::with_capacity(self.peaks.len());
            for peak in self.peaks.iter() {
                let distance = match &self.glucose_transformer {
                    None => Some(f64::abs(peak.retention_point.x() - time)),
                    Some(spline) => {
                        let maybe_gu = spline.evaluate(peak.retention_point.x());
                        maybe_gu.map(|gu| f64::abs(gu - time))
                    }
                };

                distances.push(distance);
            }
            matrix.push(distances);
        }

        let mut known = vec![];
        let mut found = 0;
        while found < self.lipid_references.len() {
            let mut closest_lipid = 0;
            let mut closest_peak = 0;
            let mut closest_distance = None;
            for reference in 0..self.lipid_references.len() {
                for peak in 0..self.peaks.len() {
                    let potential_distance = matrix[reference][peak];
                    match (closest_distance, potential_distance) {
                        (None, None) => {}
                        (None, Some(_)) => {
                            closest_distance = potential_distance;
                            closest_lipid = reference;
                            closest_peak = peak;
                        }
                        (Some(_), None) => {}
                        (Some(current), Some(potential)) => {
                            if potential < current {
                                closest_distance = potential_distance;
                                closest_lipid = reference;
                                closest_peak = peak;
                            }
                        }
                    }
                }
            }

            if closest_distance.is_none() || closest_distance.unwrap() > tolerance {
                break;
            }

            let peak = &mut self.peaks[closest_peak];
            peak.peak_type = PeakType::Common(self.lipid_references[closest_lipid].clone());
            known.push(peak.clone());

            matrix[closest_lipid] = vec![None; self.peaks.len()];
            for lipid in 0..matrix.len() {
                matrix[lipid][closest_peak] = None;
            }

            found += 1;
        }

        let mut combined = Vec::with_capacity(self.lipid_references.len());
        for reference in self.lipid_references.iter() {
            let maybe_pair = known.iter().find(|peak| match &peak.peak_type {
                PeakType::Common(potential) => reference == potential,
                _ => false,
            });

            match maybe_pair {
                Some(pair) => {
                    combined.push(pair.clone());
                }
                None => {
                    let mut missing = Peak::default();
                    missing.peak_type = PeakType::Missing(reference.clone());
                    combined.push(missing);
                }
            }
        }

        combined.sort_by(|left, right| {
            let left_rt = match &left.peak_type {
                PeakType::Common(reference) => reference.retention_time,
                PeakType::Missing(reference) => reference.retention_time,
                _ => panic!(""),
            };

            let right_rt = match &right.peak_type {
                PeakType::Common(reference) => reference.retention_time,
                PeakType::Missing(reference) => reference.retention_time,
                _ => panic!(""),
            };

            left_rt.unwrap().total_cmp(&right_rt.unwrap())
        });
        combined
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

        let iter = if self.include_unknowns {
            &self.peaks
        } else {
            &self.lipids
        };

        for peak in iter {
            let mut retention_time = format!("{:.2}", peak.retention_point.x());
            let area = format!("{:.2}", peak.area);

            let mut glucose_units = match &peak.peak_type {
                PeakType::Missing(_) => "0.00".to_string(),
                _ => {
                    let value = self.glucose_transformer.as_ref().map_or(0.0, |function| {
                        function.evaluate(peak.retention_point.x()).unwrap()
                    });
                    format!("{:.2}", value)
                }
            };

            let concentration = format!("{:.2}", peak.area * concentration_multiplier);

            let name = match &peak.peak_type {
                PeakType::Unknown => "Unknown",
                PeakType::Common(reference) => {
                    if let Some(time) = reference.retention_time {
                        retention_time.push_str(&format!("/{:.2}", time));
                    }

                    if let Some(gu) = reference.glucose_units {
                        glucose_units.push_str(&format!("/{:.2}", gu));
                    }

                    reference.name.as_ref().map_or("[Unnamed]", |inner| &inner)
                }
                PeakType::Missing(reference) => {
                    if let Some(time) = reference.retention_time {
                        retention_time.push_str(&format!("/{:.2}", time));
                    }

                    if let Some(gu) = reference.glucose_units {
                        glucose_units.push_str(&format!("/{:.2}", gu));
                    }

                    reference.name.as_ref().map_or("[Unnamed]", |inner| &inner)
                }
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
