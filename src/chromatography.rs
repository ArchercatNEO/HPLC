use std::iter::Iterator;
use std::ops::Range;

use iced::color;
use iced::widget::{container, row, scrollable, text};
use iced::{Element, Point, widget::column};
use plotters::style::BLACK;

use crate::peak::{Peak, PeakType};
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
    lipid_master_table: Vec<(f32, String)>,
    data: Vec<Point2D>,
    pub first_derivative: Vec<Point2D>,
    pub second_derivative: Vec<Point2D>,
    pub baseline: Vec<Point2D>,
    pub peaks: Vec<Peak>,
    pub lipids: Vec<Peak>,

    data_range: Option<Range<f32>>,
    include_unknowns: bool,
    height_requirement: f32,
    derivative_cone: f32,
    inflection_requirement: f32,
    horizontal_deviation: f32,
    sample_type: SampleType,

    subtract_blank: bool,
    blank_data: Option<Vec<Point2D>>,
    standard_peak: Option<Peak>,

    pub total_area: f32,

    //whyyyyyyyyyyyyyyyyyy
    //TODO: how can we not put data that is only for rendering here?
    pub title: Option<String>,
    pub global_zoom: Point,
    pub show_derivative: bool,
}

impl Chromatography {
    pub fn get_data(&self) -> Vec<Point2D> {
        if let Some(range) = &self.data_range {
            let cloned = self.data.to_vec().into_iter();
            let filtered = cloned.filter(|point| range.start < point.x() && point.x() < range.end);

            if self.subtract_blank {
                if let Some(blank) = &self.blank_data {
                    let data = filtered
                        .zip(blank)
                        .map(|(data, noise)| Point2D::new(data.x(), data.y() - noise.y()))
                        .collect();
                    return data;
                }
            }

            let vec = filtered.collect();
            Self::mean_filter(vec, 5)
        } else {
            self.data.to_vec()
        }
    }

    pub fn set_data(&mut self, value: Vec<Point2D>) -> &mut Self {
        self.data = value;
        self.first_derivative = Self::calculate_derivative(&self.get_data());
        self.second_derivative = Self::calculate_derivative(&self.first_derivative);
        self.baseline = self.calculate_baseline();
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn get_data_range(&self) -> Range<f32> {
        if let Some(range) = &self.data_range {
            range.clone()
        } else {
            let default = &Point2D::default();
            let end = self.data.last().unwrap_or(default);
            0.0..end.x()
        }
    }

    /// When performing HPLC there may be extreme noise
    /// at the beginning and end of the sample.
    /// Setting this will crop the raw data for the purposes of the baseline and peaks
    pub fn set_data_range(&mut self, value: Range<f32>) -> &mut Self {
        self.data_range = Some(value);
        self.first_derivative = Self::calculate_derivative(&self.get_data());
        self.second_derivative = Self::calculate_derivative(&self.first_derivative);
        self.baseline = self.calculate_baseline();
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn get_highest_point(&self) -> f32 {
        let data = self.get_data();
        if data.len() == 0 {
            return 0.0;
        }

        let mut highest = 0.0;
        for point in data {
            if point.y() > highest {
                highest = point.y();
            }
        }

        highest
    }

    pub fn set_lipid_master_table(&mut self, value: Vec<(f32, String)>) -> &mut Self {
        self.lipid_master_table = value;
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_include_unknowns(&mut self, show: bool) -> &mut Self {
        self.include_unknowns = show;
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_height_requirement(&mut self, value: f32) -> &mut Self {
        self.height_requirement = value;
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_derivative_cone(&mut self, value: f32) -> &mut Self {
        self.derivative_cone = value;
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_inflection_requirement(&mut self, value: f32) -> &mut Self {
        self.inflection_requirement = value;
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_horizontal_deviation(&mut self, value: f32) -> &mut Self {
        self.horizontal_deviation = value;
        self.lipids = self.label_peaks();

        self
    }

    pub fn get_sample_type(&self) -> SampleType {
        self.sample_type
    }

    pub fn set_sample_type(&mut self, value: SampleType) -> &mut Self {
        self.sample_type = value;

        self
    }

    pub fn set_subtract_blank(&mut self, value: bool) -> &mut Self {
        self.subtract_blank = value;
        self.baseline = self.calculate_baseline();
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_blank_data(&mut self, value: Option<Vec<Point2D>>) -> &mut Self {
        self.blank_data = value;
        self.baseline = self.calculate_baseline();
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_standard_peak(&mut self, value: Option<Peak>) -> &mut Self {
        self.standard_peak = value;
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    fn mean_filter(data: Vec<Point2D>, smoothing: isize) -> Vec<Point2D> {
        let mut smoothed = Vec::with_capacity(data.len());

        let isize_len = data.len() as isize;
        for i in 0..isize_len {
            let mut total = 0.0;
            let start = isize::max(0, i - smoothing);
            let end = isize::min(isize_len, i + smoothing);

            for offset in start..end {
                total += data[offset as usize].y();
            }

            let point = Point2D::new(data[i as usize].x(), total / (end - start) as f32);
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
        let data = self.get_data();
        if data.len() < 2 {
            return vec![];
        }

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
        let data = self.get_data();
        if data.len() == 0 {
            return vec![];
        }

        self.total_area = 0.0;

        let mut result = vec![];

        let mut peak = Peak::default();
        peak.start = data[0].clone();

        let mut new_peak = true;
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
                    if let Some(standard) = &self.standard_peak {
                        peak.concentration = peak.area * standard.area * 40.0 * 20.0 * 0.0025;
                    }

                    result.push(peak);
                    peak = Peak::default();
                    peak.start = prev.clone();
                } else if height < prev_min {
                    // Lower minimum but without a peak, merge with prev peak
                    prev_min = height;

                    if let Some(prev_peak) = result.last_mut() {
                        prev_peak.end = prev.clone();
                        prev_peak.area += peak.area;
                        if let Some(standard) = &self.standard_peak {
                            prev_peak.concentration =
                                prev_peak.area * standard.area * 40.0 * 20.0 * 0.0025;
                        }
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
                peak.peak_type = PeakType::Shoulder(BLACK);
                peak.retention_point = next.clone();
                peak.end = next.clone();
                if let Some(standard) = &self.standard_peak {
                    peak.concentration = peak.area * standard.area * 40.0 * 20.0 * 0.0025;
                }
                
                result.push(peak);
                
                peak = Peak::default();
                peak.start = next.clone();
            }

            let falling_zero = prev_drv2.y() >= 0.0 && next_drv2.y() <= 0.0;

            if falling_zero && prev_drv.y() <= 0.0 {
                peak.end = next.clone();
                if let Some(standard) = &self.standard_peak {
                    peak.concentration = peak.area * standard.area * 40.0 * 20.0 * 0.0025;
                }
                
                result.push(peak);
                
                peak = Peak::default();
                peak.start = next.clone();
                peak.retention_point = next.clone();
                peak.peak_type = PeakType::Shoulder(BLACK);
            }
        }

        result
    }

    fn label_peaks(&mut self) -> Vec<Peak> {
        let mut known: Vec<Peak> = vec![];

        for peak in self.peaks.iter_mut() {
            peak.lipid = None;
        }

        for (retention_time, lipid) in self.lipid_master_table.iter() {
            for i in 1..self.peaks.len() {
                let prev = &self.peaks[i - 1];
                let next = &self.peaks[i];

                // If we are at the first peak, hope it's a relevant peak
                if i == 1
                    && f32::abs(prev.retention_point.x() - *retention_time)
                        < self.horizontal_deviation
                {
                    let mut peak = prev.clone();
                    peak.lipid = Some(lipid.clone());
                    self.peaks[i - 1].lipid = Some(lipid.clone());
                    known.push(peak);
                    break;
                }

                if prev.retention_point.x() <= *retention_time
                    && *retention_time <= next.retention_point.x()
                {
                    let dist1 = retention_time - prev.retention_point.x();
                    let dist2 = next.retention_point.x() - retention_time;

                    if dist1 < dist2 && dist1 < self.horizontal_deviation {
                        let mut peak = prev.clone();
                        peak.lipid = Some(lipid.clone());
                        self.peaks[i - 1].lipid = Some(lipid.clone());
                        known.push(peak);
                    } else if dist2 < self.horizontal_deviation {
                        let mut peak = next.clone();
                        peak.lipid = Some(lipid.clone());
                        self.peaks[i].lipid = Some(lipid.clone());
                        known.push(peak);
                    } else {
                        let mut fake = Peak::default();
                        fake.lipid = Some(lipid.clone());
                        known.push(fake);
                    }

                    break;
                }
            }
        }

        known
    }

    pub fn into_table_csv(&self) -> String {
        self.peaks
            .iter()
            .filter(|peak| self.include_unknowns || peak.lipid != None)
            .map(|peak| {
                let entry = peak.lipid.clone().unwrap_or("Unknown".to_string());
                format!("{},{},{}\n", entry, peak.retention_point.x(), peak.area)
            })
            .fold(
                "Lipid,Retention Time (m),Area\n".to_string(),
                |accum, entry| accum + &entry,
            )
    }

    pub fn into_table_element<'a>(&'a self) -> Element<'a, ()> {
        let mut table = column![];
        let title = text(format!("Total Area - {}", self.total_area))
            .center()
            .width(750);

        let mut gray = container::Style::default();
        gray = gray.background(color!(0xaaaaaa));

        let lipid_label = text("Lipid").center().width(200);
        let retention_label = text("Retention Time (m)").center().width(200);
        let area_label = text("Area").center().width(150);
        let concentration_label = text("Concentration (nmol/ml)").center().width(200);

        let header = row![
            text("|"),
            container(lipid_label).style(move |_| gray),
            text("|"),
            container(retention_label).style(move |_| gray),
            text("|"),
            container(area_label).style(move |_| gray),
            text("|"),
            container(concentration_label).style(move |_| gray),
            text("|"),
        ]
        .spacing(20);

        let spacer_string = "-".repeat(175);

        table = table.push(title);
        table = table.push(text(spacer_string.clone()));
        table = table.push(header);

        let iter = if self.include_unknowns {
            &self.peaks
        } else {
            &self.lipids
        };

        for peak in iter {
            let name = peak.lipid.as_ref().map_or("Unknown", |s| &s);
            let retention_time = crate::round_to_precision(peak.retention_point.x(), 2);
            let area = crate::round_to_precision(peak.area, 2);
            let concentration = crate::round_to_precision(peak.concentration, 2);
            let content = row![
                text("|"),
                text(name).center().width(200),
                text("|"),
                text(retention_time).center().width(200),
                text("|"),
                text(area).center().width(150),
                text("|"),
                text(format!("{}", concentration)).center().width(200),
                text("|"),
            ]
            .spacing(20);

            table = table.push(text(spacer_string.clone()));
            table = table.push(content);
        }

        table = table.push(text(spacer_string.clone()));
        scrollable(table).into()
    }
}
