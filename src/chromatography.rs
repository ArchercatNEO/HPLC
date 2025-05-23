use std::iter::Iterator;
use std::ops::Range;

use iced::widget::{row, scrollable, text};
use iced::{Element, Point, widget::column};

use crate::peak::Peak;
use crate::vector::*;

#[derive(Clone, Debug, Default)]
pub struct Chromatography {
    lipid_master_table: Vec<(f32, String)>,
    data: Vec<Point2D>,
    pub baseline: Vec<Point2D>,
    pub peaks: Vec<Peak>,
    pub lipids: Vec<Peak>,

    data_range: Option<Range<f32>>,
    include_unknowns: bool,
    noise_reduction: f32,
    horizontal_deviation: f32,

    //whyyyyyyyyyyyyyyyyyy
    //TODO: how can we not put data that is only for rendering here?
    pub global_zoom: Point,
}

impl Chromatography {
    pub fn get_data(&self) -> Vec<Point2D> {
        if let Some(range) = &self.data_range {
            let cloned = self.data.to_vec().into_iter();
            let filtered = cloned.filter(|point| range.start < point.x() && point.x() < range.end);
            filtered.collect()
        } else {
            self.data.to_vec()
        }
    }

    pub fn set_data(&mut self, value: Vec<Point2D>) -> &mut Self {
        self.data = value;
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

    pub fn set_noise_reduction(&mut self, value: f32) -> &mut Self {
        self.noise_reduction = value;
        self.peaks = self.calculate_peaks();
        self.lipids = self.label_peaks();

        self
    }

    pub fn set_horizontal_deviation(&mut self, value: f32) -> &mut Self {
        self.horizontal_deviation = value;
        self.lipids = self.label_peaks();

        self
    }

    fn calculate_baseline(&self) -> Vec<Point2D> {
        let data = self.get_data();
        if data.len() == 0 {
            return vec![];
        }

        let mut index = 1;

        let mut origin = &data[0];
        let mut next = &Point2D::default();
        let mut baseline = vec![data[0].clone()];

        while index + 1 < data.len() {
            let mut best_gradient = f32::INFINITY;
            for i in index..data.len() {
                let point = &data[i];

                let gradient = origin.gradient(point);
                if gradient < best_gradient {
                    next = point;
                    best_gradient = gradient;
                    index = i;
                }
            }

            baseline.push(next.clone());
            origin = &next;
        }

        baseline
    }

    fn calculate_peaks(&self) -> Vec<Peak> {
        let data = self.get_data();
        if data.len() == 0 {
            return vec![];
        }

        let mut result = vec![];

        let mut baseline = self.baseline.iter();
        let mut baseline_start = baseline.next().unwrap();
        let mut baseline_end = baseline.next().unwrap();

        let mut gradient = baseline_start.gradient(baseline_end);
        let mut offset = baseline_start.y() - gradient * baseline_start.x();

        let mut peak = Peak::default();
        for point in data.iter() {
            if peak.start == Point2D::default() {
                peak.start = point.clone();
                peak.area += (point.y() - gradient * baseline_start.x() - offset) / 2.0;
                continue;
            }

            if baseline_end.x() < point.x() {
                if let Some(next) = baseline.next() {
                    baseline_start = baseline_end;
                    baseline_end = next;

                    gradient = baseline_start.gradient(baseline_end);
                    offset = baseline_start.y() - gradient * baseline_start.x();
                }
            }

            if peak.turning_point.y() < point.y() {
                peak.turning_point = point.clone();
                peak.area += point.y() - gradient * point.x() - offset;
                continue;
            }

            if peak.end == Point2D::default() || peak.end.y() > point.y() {
                peak.end = point.clone();
                peak.area += point.y() - (gradient * point.x() + offset);
            } else {
                let end = peak.end.clone();
                peak.area -= (peak.end.y() - gradient * peak.end.x() - offset) / 2.0;
                peak.area *= point.x() - peak.end.x();

                if peak.turning_point.y() - peak.start.y() > self.noise_reduction {
                    result.push(peak);
                }

                peak = Peak::default();
                peak.start = end;
            }
        }

        result
    }

    fn label_peaks(&mut self) -> Vec<Peak> {
        let mut known: Vec<Peak> = vec![];

        for (retention_time, lipid) in self.lipid_master_table.iter() {
            for i in 1..self.peaks.len() {
                let prev = &self.peaks[i - 1];
                let next = &self.peaks[i];

                if prev.turning_point.x() < *retention_time
                    && *retention_time < next.turning_point.x()
                {
                    let dist1 = retention_time - prev.turning_point.x();
                    let dist2 = next.turning_point.x() - retention_time;

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

                        self.peaks[i - 1].lipid = None;
                        self.peaks[i].lipid = None;
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
                format!("{},{},{}\n", entry, peak.turning_point.x(), peak.area)
            })
            .fold(
                "Lipid,Retention Time (s),Area\n".to_string(),
                |accum, entry| accum + &entry,
            )
    }

    //TODO: use `self.peaks` when `include_unknowns` is true
    pub fn into_table_element<'a>(&'a self) -> Element<'a, ()> {
        let mut table = column![];
        let header = row![
            text("Lipid").center().width(200),
            text("Retention Time (s)").center().width(200),
            text("Area").center().width(150)
        ]
        .spacing(20);

        table = table.push(header);
        for lipid in &self.lipids {
            let name = lipid.lipid.clone().unwrap();
            let retention_time = crate::round_to_precision(lipid.turning_point.x(), 2);
            let area = crate::round_to_precision(lipid.area, 2);
            let content = row![
                text(name).center().width(200),
                text(retention_time).center().width(200),
                text(area).center().width(150)
            ]
            .spacing(20);
            table = table.push(content);
        }

        scrollable(table).into()
    }
}
