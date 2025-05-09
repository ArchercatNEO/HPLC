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
    fn clamp_vec(data: &[Point2D], range: &Option<Range<f32>>) -> Vec<Point2D> {
        if let Some(range) = range {
            let cloned = data.to_vec().into_iter();
            let filtered = cloned.filter(|point| range.contains(&point.x()));
            filtered.collect()
        } else {
            data.to_vec()
        }
    }

    pub fn get_data(&self) -> Vec<Point2D> {
        Self::clamp_vec(&self.data, &self.data_range)
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
                let point = &self.data[i];

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

        let mut peak = Peak::default();
        for point in data.iter() {
            if peak.start == Point2D::default() {
                peak.start = point.clone();
                continue;
            }

            if peak.turning_point.y() < point.y() {
                peak.turning_point = point.clone();
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
            } else {
                if peak.turning_point.y() - peak.start.y() > self.noise_reduction {
                    result.push(peak);
                }

                peak = Peak::default();
                peak.start = point.clone();
            }
        }

        result
    }
}
