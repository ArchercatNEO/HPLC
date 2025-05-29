use std::iter::Iterator;
use std::ops::Range;

use iced::color;
use iced::widget::{container, row, scrollable, text};
use iced::{Element, Point, widget::column};

use crate::peak::Peak;
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
    pub baseline: Vec<Point2D>,
    pub peaks: Vec<Peak>,
    pub lipids: Vec<Peak>,

    data_range: Option<Range<f32>>,
    include_unknowns: bool,
    noise_reduction: f32,
    horizontal_deviation: f32,
    sample_type: SampleType,

    subtract_blank: bool,
    blank_data: Option<Vec<Point2D>>,
    //dex_mapper: Box<dyn Fn(f32) -> f32>,
    standard_peak: Option<Peak>,

    pub total_area: f32,

    //whyyyyyyyyyyyyyyyyyy
    //TODO: how can we not put data that is only for rendering here?
    pub title: Option<String>,
    pub global_zoom: Point,
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
        let mut index = 0;
        while index + 1 < data.len() {
            index += 1;
            let mut prev = &data[index - 1];
            let mut next = &data[index];

            let area = {
                let h = next.x() - prev.x();
                let a = prev.y() - self.baseline[index - 1].y();
                let b = next.y() - self.baseline[index].y();
                h * (a + b) / 2.0
            };

            self.total_area += area;
            peak.area += area;

            if peak.start == Point2D::default() {
                peak.start = prev.clone();
                continue;
            }

            while prev.y() < next.y() {
                index += 1;
                if index == data.len() {
                    return result;
                }

                prev = &data[index - 1];
                next = &data[index];

                let area = {
                    let h = next.x() - prev.x();
                    let a = prev.y() - self.baseline[index - 1].y();
                    let b = next.y() - self.baseline[index].y();
                    h * (a + b) / 2.0
                };

                self.total_area += area;
                peak.area += area;
            }

            peak.turning_point = prev.clone();

            while prev.y() > next.y() {
                index += 1;
                if index == data.len() {
                    return result;
                }

                prev = &data[index - 1];
                next = &data[index];

                let area = {
                    let h = next.x() - prev.x();
                    let a = prev.y() - self.baseline[index - 1].y();
                    let b = next.y() - self.baseline[index].y();
                    h * (a + b) / 2.0
                };

                self.total_area += area;
                peak.area += area;
            }

            peak.end = prev.clone();
            if peak.turning_point.y() - peak.start.y() > self.noise_reduction {
                if let Some(standard) = &self.standard_peak {
                    peak.concentration = peak.area * standard.area * 40.0 * 20.0 * 0.0025;
                }

                result.push(peak);
            }

            peak = Peak::default();
            peak.start = prev.clone();
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
                    && f32::abs(prev.turning_point.x() - *retention_time)
                        < self.horizontal_deviation
                {
                    let mut peak = prev.clone();
                    peak.lipid = Some(lipid.clone());
                    self.peaks[i - 1].lipid = Some(lipid.clone());
                    known.push(peak);
                    break;
                }

                if prev.turning_point.x() <= *retention_time
                    && *retention_time <= next.turning_point.x()
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
            let retention_time = crate::round_to_precision(peak.turning_point.x(), 2);
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
