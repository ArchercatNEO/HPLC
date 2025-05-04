use iced::{Element, Length};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

use crate::analysis::Peak;

#[derive(Debug, Default)]
pub struct HplcChart {
    data: Vec<(f32, f32)>,
    baseline: Vec<(f32, f32)>,
    turning_points: Vec<(f32, f32)>,
    pub noise_tolerance: f32,
}

#[derive(Clone, Debug)]
pub enum Message {}

impl HplcChart {
    pub fn view(&self) -> Element<Message> {
        let chart = ChartWidget::new(self)
            .height(Length::Fill)
            .width(Length::Fill);

        chart.into()
    }

    pub fn update(&mut self, _message: Message) {}

    pub fn set_chart_data(&mut self, data: Vec<(f32, f32)>) {
        let truncated = data.into_iter().filter(|(x, _)| 9.0 < *x && *x < 35.0);

        self.data = truncated.clone().collect();

        let mut origin = self.data[0];
        let mut next = (0.0, 0.0);
        let mut index = 0;
        let mut baseline = vec![origin];
        
        while index + 1 < self.data.len() {
            let mut gradient = f32::INFINITY;
                        
            for i in index..self.data.len() {
                let point = &self.data[i];
                let delta_x = point.0 - origin.0;
                let delta_y = point.1 - origin.1;
                let new_gradient = delta_y / delta_x;
                
                if new_gradient < gradient {
                    gradient = new_gradient;
                    next = point.clone();
                    index = i;
                }
            };

            baseline.push(next);
            origin = next;
        }

        self.baseline = baseline;

        self.turning_points = Peak::find_peaks(truncated, self.noise_tolerance)
            .iter()
            .flat_map(|peak| [peak.start, peak.turning_point, peak.end])
            .collect();
    }

    pub fn set_noise_tolerance(&mut self, value: f32) {
        self.noise_tolerance = value;

        self.turning_points = Peak::find_peaks(self.data.clone(), self.noise_tolerance)
            .iter()
            .flat_map(|peak| [peak.start, peak.turning_point, peak.end])
            .collect();
    }
}

impl Chart<Message> for HplcChart {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        let mut chart = builder
            .caption("HPLC", ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(9f32..35f32, -2f32..150f32)
            .expect("failed to build chart");

        chart
            .configure_mesh()
            .draw()
            .expect("failed to configure chart");

        chart
            .draw_series(LineSeries::new(self.data.clone(), &RED))
            .expect("failed to draw series")
            .label("data")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .draw_series(LineSeries::new(self.baseline.clone(), &GREEN))
            .expect("failed to draw series")
            .label("baseline")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));

        let peak_points: PointSeries<
            '_,
            (f32, f32),
            Vec<(f32, f32)>,
            Circle<(f32, f32), i32>,
            i32,
        > = PointSeries::new(self.turning_points.clone(), 5, &BLUE);

        chart
            .draw_series(peak_points)
            .expect("failed to draw series")
            .label("data")
            .legend(|(x, y)| Circle::new((x, y), 5, &BLUE));
    }
}
