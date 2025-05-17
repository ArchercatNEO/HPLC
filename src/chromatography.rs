use std::iter::Iterator;
use std::ops::Range;

use crate::peak::Peak;
use crate::vector::*;

#[derive(Clone, Debug, Default)]
pub struct Chromatography {
    data: Vec<Point2D>,

    data_range: Option<Range<f32>>,
    pub baseline: Vec<Point2D>,

    pub noise_reduction: f32,
    lipid_master_table: Vec<(f32, String)>,
    pub peaks: Vec<Peak>,
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

    pub fn set_noise_reduction(&mut self, value: f32) -> &mut Self {
        self.noise_reduction = value;
        self.peaks = self.calculate_peaks();

        self
    }

    pub fn set_lipid_master_table(&mut self, value: Vec<(f32, String)>) -> &mut Self {
        self.lipid_master_table = value;
        self.peaks = self.calculate_peaks();

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

        let mut lipids = self.lipid_master_table.iter();
        let mut lipid = lipids.next();

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
            } else if peak.lipid == None {
                if let Some((x, name)) = lipid {
                    if f32::abs(peak.turning_point.x() - x) < 0.5 {
                        peak.lipid = Some(name.clone());
                        lipid = lipids.next();
                    }
                }
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
}
