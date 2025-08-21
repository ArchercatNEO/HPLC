use std::ffi::OsString;
use std::fs;
use std::iter::Iterator;
use std::ops::Range;
use std::path::Path;
use std::rc::Rc;

use iced::color;
use iced::widget::{container, row, scrollable, text};
use iced::{Element, Point, widget::column};

use crate::cubic::Cubic;
use crate::peak::{Peak, PeakType};
use crate::reference::Reference;
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
    pub total_area: f32,

    // Chromatography configuration
    sample_type: SampleType,
    data_range: Option<Range<f32>>,
    include_unknowns: bool,
    height_requirement: f32,
    inflection_requirement: f32,
    retention_time_tolerance: f32,
    glucose_unit_tolerance: f32,

    // External references
    lipid_references: Rc<[Reference]>,
    glucose_transformer: Option<Cubic>,

    // Display configuration + state
    //TODO: how can we not put data that is only for rendering here?
    pub title: String,
    pub file_name: OsString,
    pub global_zoom: Point,
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

            for line in file.lines() {
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
            .lines()
            .filter_map(|line| {
                let mut data = if line.contains("\t") {
                    line.split("\t")
                } else {
                    line.split(",")
                };

                let x: f32 = match data.next() {
                    Some(string) => match string.parse::<f32>() {
                        Ok(value) => value,
                        Err(_) => return None,
                    },
                    None => return None,
                };

                let y: f32 = match data.next() {
                    Some(string) => match string.parse::<f32>() {
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
        empty.baseline = empty.calculate_baseline();
        empty.peaks = empty.calculate_peaks();
        empty.lipids = empty.label_peaks();

        Some(empty)
    }

    pub fn get_data(&self) -> Vec<Point2D> {
        self.cleaned_data.clone()
    }

    pub fn get_data_range(&self) -> Range<f32> {
        if let Some(range) = &self.data_range {
            range.clone()
        } else {
            let default = &Point2D::default();
            let end = self.raw_data.last().unwrap_or(default);
            0.0..end.x()
        }
    }

    pub fn set_data_range(&mut self, value: &Range<f32>) -> &mut Self {
        let filter: Vec<Point2D> = self
            .raw_data
            .iter()
            .cloned()
            .filter(|point| value.contains(&point.x()))
            .collect();

        self.data_range = Some(value.clone());
        self.cleaned_data = Self::mean_filter(&filter, 5);
        self.first_derivative = Self::calculate_derivative(&self.cleaned_data);
        self.second_derivative = Self::calculate_derivative(&self.first_derivative);
        self.baseline = self.calculate_baseline();
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn get_highest_point(&self) -> f32 {
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
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_include_unknowns(&mut self, show: &bool) -> &mut Self {
        self.include_unknowns = *show;
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_height_requirement(&mut self, value: &f32) -> &mut Self {
        self.height_requirement = *value;
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_inflection_requirement(&mut self, value: &f32) -> &mut Self {
        self.inflection_requirement = *value;
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_retention_time_tolerance(&mut self, value: &f32) -> &mut Self {
        self.retention_time_tolerance = *value;
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_glucose_unit_tolerance(&mut self, value: &f32) -> &mut Self {
        self.glucose_unit_tolerance = *value;
        self.lipids = self.label_peaks();

        self
    }

    pub fn get_glucose_transformer(&self) -> Cubic {
        let mut intersections: Vec<Point2D> = vec![];

        for peak in &self.peaks {
            let retention = peak.retention_point.clone();
            intersections = intersections
                .into_iter()
                .filter(|prev| prev.y() > retention.y())
                .collect();
            intersections.push(retention);
        }

        // Method of least squares (cubic interpolation)
        // f(x) = ax^3 + bx^2 + cx + d
        let x0 = intersections.len() as f32;
        let x1: f32 = intersections.iter().map(|point| point.x()).sum();
        let x2: f32 = intersections.iter().map(|point| point.x().powi(2)).sum();
        let x3: f32 = intersections.iter().map(|point| point.x().powi(3)).sum();
        let x4: f32 = intersections.iter().map(|point| point.x().powi(4)).sum();
        let x5: f32 = intersections.iter().map(|point| point.x().powi(5)).sum();
        let x6: f32 = intersections.iter().map(|point| point.x().powi(6)).sum();

        let y1: f32 = intersections
            .iter()
            .enumerate()
            .map(|(index, _)| (index + 2) as f32)
            .sum();
        let y1x1: f32 = intersections
            .iter()
            .enumerate()
            .map(|(index, point)| ((index + 2) as f32) * point.x())
            .sum();
        let y1x2: f32 = intersections
            .iter()
            .enumerate()
            .map(|(index, point)| ((index + 2) as f32) * point.x().powi(2))
            .sum();
        let y1x3: f32 = intersections
            .iter()
            .enumerate()
            .map(|(index, point)| ((index + 2) as f32) * point.x().powi(3))
            .sum();

        let mut matrix = [
            vec![x6, x5, x4, x3],
            vec![x5, x4, x3, x2],
            vec![x4, x3, x2, x1],
            vec![x3, x2, x1, x0],
        ];

        let mut values = [y1x3, y1x2, y1x1, y1];

        let solution = Self::solve_matrix(&mut matrix, &mut values).unwrap();

        let a = solution[0];
        let b = solution[1];
        let c = solution[2];
        let d = solution[3];

        let function = Cubic::new(a, b, c, d);

        function
    }

    fn solve_matrix(matrix: &mut [Vec<f32>], values: &mut [f32]) -> Option<Vec<f32>> {
        let order = matrix.len();

        for i in 0..order {
            // Partial pivoting
            let mut max_row = i;
            for k in (i + 1)..order {
                if matrix[k][i].abs() > matrix[max_row][i].abs() {
                    max_row = k;
                }
            }

            // Swap rows in matrix and vector
            matrix.swap(i, max_row);
            values.swap(i, max_row);

            // Check for singular matrix
            if matrix[i][i].abs() < 1e-12 {
                return None; // Singular or nearly singular matrix
            }

            // Eliminate entries below pivot
            for k in (i + 1)..order {
                let factor = matrix[k][i] / matrix[i][i];
                for j in i..order {
                    matrix[k][j] -= factor * matrix[i][j];
                }
                values[k] -= factor * values[i];
            }
        }

        // Back substitution
        let mut x = vec![0.0; order];
        for i in (0..order).rev() {
            let mut sum = values[i];
            for j in (i + 1)..order {
                sum -= matrix[i][j] * x[j];
            }
            x[i] = sum / matrix[i][i];
        }

        Some(x)
    }

    pub fn set_glucose_transformer(&mut self, transformer: &Option<Cubic>) -> &mut Self {
        self.glucose_transformer = transformer.clone();
        self.lipids = self.label_peaks();

        self
    }

    pub fn get_sample_type(&self) -> SampleType {
        self.sample_type
    }

    pub fn set_sample_type(&mut self, value: &SampleType) -> &mut Self {
        self.sample_type = value.clone();

        self
    }

    fn mean_filter(data: &[Point2D], smoothing: usize) -> Vec<Point2D> {
        let mut smoothed = Vec::with_capacity(data.len());

        for i in 0..data.len() {
            let mut total = 0.0;
            let start = i.saturating_sub(smoothing);
            let end = (i + smoothing).min(data.len() - 1);

            for offset in start..end {
                total += data[offset].y();
            }

            let point = Point2D::new(data[i].x(), total / (end - start) as f32);
            smoothed.push(point);
        }

        smoothed
    }

    fn calculate_derivative(graph: &[Point2D]) -> Vec<Point2D> {
        if graph.len() < 2 {
            return vec![];
        }

        let mut derivative = Vec::with_capacity(graph.len());

        for i in 1..(graph.len() - 1) {
            let prev = &graph[i - 1];
            let next = &graph[i + 1];

            let point = Point2D::new(prev.x(), prev.gradient(next));
            derivative.push(point);
        }

        derivative
    }

    fn calculate_baseline(&self) -> Vec<Point2D> {
        let data = &self.cleaned_data;

        let mut origin = &data[0];
        let mut orgin_index = 0;

        let mut next = &Point2D::default();
        let mut next_index = 1;

        let mut baseline = vec![];

        while next_index + 1 < data.len() {
            let mut best_gradient = f32::INFINITY;
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

    fn calculate_peaks(&mut self) -> Vec<Peak> {
        let data = &self.cleaned_data;

        self.total_area = 0.0;

        let mut result = vec![];

        let mut peak = Peak::default();
        peak.start = data[0].clone();

        let mut new_peak = false;
        let mut prev_min = data[0].y() - self.baseline[0].y();

        for index in 4..(data.len() - 4) {
            let prev = &data[index - 1];
            let next = &data[index];

            let height = prev.y() - self.baseline[index - 1].y();

            let area = {
                let h = next.x() - prev.x();
                let a = prev.y() - self.baseline[index - 1].y();
                let b = next.y() - self.baseline[index].y();
                h * (a + b) / 2.0
            };

            self.total_area += area;
            peak.area += area;

            let prev_drv = &self.first_derivative[index - 2];
            let next_drv = &self.first_derivative[index - 1];

            if prev_drv.y() <= 0.0 && next_drv.y() >= 0.0 {
                // Minimum
                if new_peak {
                    // Real peak
                    prev_min = height;

                    peak.end = prev.clone();

                    result.push(peak);
                    peak = Peak::default();
                    peak.start = prev.clone();
                } else if height < prev_min {
                    // Lower minimum but without a peak, merge with prev peak
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
                new_peak = height > self.height_requirement;
                peak.retention_point = prev.clone();
                peak.height = height;
            }

            let prev_drv2 = &self.second_derivative[index - 3];
            let next_drv2 = &self.second_derivative[index - 2];

            let difference = f32::abs(prev_drv2.y() - next_drv2.y());

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

        result
    }

    fn label_peaks(&mut self) -> Vec<Peak> {
        let mut known: Vec<Peak> = vec![];

        let transformer = match self.glucose_transformer {
            Some(cubic) => cubic,
            None => Cubic::default(),
        };

        for peak in self.peaks.iter_mut() {
            peak.peak_type = PeakType::Unknown;
            peak.gu = transformer.evaluate(peak.retention_point.x())
        }

        let tolerance = match self.glucose_transformer {
            Some(_) => self.glucose_unit_tolerance,
            None => self.retention_time_tolerance,
        };

        for reference in self.lipid_references.iter() {
            for i in 1..self.peaks.len() {
                let (left, right) = self.peaks.split_at_mut(i);
                let prev = left.last_mut().unwrap();
                let next = &mut right[0];

                let start = transformer.evaluate(prev.retention_point.x());
                let end = transformer.evaluate(next.retention_point.x());

                let expected = if let Some(gu) = reference.glucose_units {
                    if self.glucose_transformer.is_some() {
                        gu
                    } else {
                        reference.retention_time.unwrap()
                    }
                } else if let Some(retention) = reference.retention_time {
                    transformer.evaluate(retention)
                } else {
                    continue;
                };

                // A lipid may be expected before the first peak (and is therefore not between 2 peaks)
                if f32::abs(expected - start) < tolerance {
                    prev.peak_type = PeakType::Common(reference.clone());
                    known.push(prev.clone());
                    break;
                }

                // If a lipid is expected between 2 peaks choose the closer one
                if (start..end).contains(&expected) {
                    let dist1 = expected - start;
                    let dist2 = end - expected;

                    if dist1 < dist2 && dist1 < tolerance {
                        prev.peak_type = PeakType::Common(reference.clone());
                        known.push(prev.clone());
                    } else if dist2 < tolerance {
                        next.peak_type = PeakType::Common(reference.clone());
                        known.push(next.clone());
                    } else {
                        let mut fake = Peak::default();
                        fake.peak_type = PeakType::Missing(reference.clone());
                        known.push(fake);
                    }

                    break;
                }
            }
        }

        known
    }

    pub fn set_global_zoom(&mut self, zoom: &Point) -> &mut Self {
        self.global_zoom = zoom.clone();

        self
    }

    pub fn into_table_element<'b>(&'b self, concentration_multiplier: f32) -> Element<'b, ()> {
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
                    let value = self
                        .glucose_transformer
                        .map_or(0.0, |function| function.evaluate(peak.retention_point.x()));
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
