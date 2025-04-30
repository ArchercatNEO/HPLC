use std::path;

use iced::{
    Element, Length, Task,
    widget::{button, row},
};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};
use rfd::AsyncFileDialog;

use crate::peak::Peak;

#[derive(Debug, Default)]
pub struct HplcChart {
    data: Vec<(f32, f32)>,
    turning_points: Vec<(f32, f32)>,
}

#[derive(Clone, Debug)]
pub enum Message {
    ChartLoad,
    ChartLoadCancel,
    ChartLoaded(Vec<(f32, f32)>),
}

impl HplcChart {
    pub fn view(&self) -> Element<Message> {
        let chart = ChartWidget::new(self)
            .height(Length::Fixed(1000.0))
            .width(Length::Fixed(1000.0));

        let load_file = button("Load File").on_press(Message::ChartLoad);

        row![chart, load_file].into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChartLoad => {
                let task = AsyncFileDialog::new()
                    .add_filter("text", &["tsv"])
                    .pick_file();

                Task::perform(task, |maybe_file| {
                    let maybe_path = maybe_file.map(|file| {
                        let path = file.path();
                        parse_file(path)
                    });

                    maybe_path.map_or(Message::ChartLoadCancel, Message::ChartLoaded)
                })
            }
            Message::ChartLoadCancel => Task::none(),
            Message::ChartLoaded(peaks) => {
                self.data = peaks.clone();
                self.turning_points = Peak::find_peaks(peaks)
                    .iter()
                    .flat_map(|peak| [peak.start, peak.turning_point, peak.end])
                    .collect();

                Task::none()
            }
        }
    }
}

impl Chart<Message> for HplcChart {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, state: &Self::State, mut builder: ChartBuilder<DB>) {
        let mut chart = builder
            .caption("HPLC", ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(5f32..50f32, -2f32..700f32)
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

fn parse_file<'a, P: AsRef<path::Path>>(path: P) -> Vec<(f32, f32)> {
    let file = std::fs::read_to_string(path).unwrap();

    let sequence = file
        .split("\n")
        .filter(|line| !line.is_empty())
        .filter_map(parse_line)
        .filter(|coord| 5.0 < coord.0 && coord.0 < 50.0);

    sequence.collect()
}

fn parse_line(line: &str) -> Option<(f32, f32)> {
    let mut data = line.split("\t");

    let x_str = data.next();
    if x_str == None {
        return None;
    }

    let x_coord = x_str.unwrap().parse();
    if let Err(_) = x_coord {
        return None;
    }

    let x = x_coord.unwrap();

    let y_str = data.next();
    if y_str == None {
        return None;
    }

    let y_coord = y_str.unwrap().parse();
    if let Err(_) = y_coord {
        return None;
    }

    let y = y_coord.unwrap();

    Some((x, y))
}
