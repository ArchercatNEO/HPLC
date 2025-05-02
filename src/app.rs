use iced::{
    Element, Task,
    alignment::Horizontal,
    widget::{button, column, row, slider, text},
};
use rfd::AsyncFileDialog;

use crate::{analysis, chart::HplcChart};

#[derive(Debug, Default)]
pub struct App {
    file_path: &'static str,
    chart: HplcChart,
}

#[derive(Clone, Debug)]
enum Message {
    Done,
    DataFileChangeRequest,
    DataLoaded(Vec<(f32, f32)>),
    NoiseToleranceChanged(f32),
    ChartMessage(crate::chart::Message),
}

impl From<crate::chart::Message> for Message {
    fn from(value: crate::chart::Message) -> Self {
        Message::ChartMessage(value)
    }
}

impl App {
    pub fn start() -> iced::Result {
        iced::run("HPLC", Self::update, Self::view)
    }

    fn view(&self) -> Element<Message> {
        let load_file = column![
            text(format!("Current File: {}", self.file_path)),
            button("Load File").on_press(Message::DataFileChangeRequest)
        ];

        let noise_tolerance = column![
            text(format!("Noise Tolerance: {}", self.chart.noise_tolerance))
                .align_x(Horizontal::Center),
            slider(
                0.0..=100.0,
                self.chart.noise_tolerance,
                Message::NoiseToleranceChanged
            )
            .width(200)
        ]
        .max_width(200);

        let options = column![load_file, noise_tolerance];

        let chart = self.chart.view();

        row![options, chart.map(Message::ChartMessage)].into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Done => Task::none(),
            Message::DataFileChangeRequest => {
                let task = AsyncFileDialog::new()
                    .add_filter("text", &["tsv"])
                    .pick_file();

                Task::perform(task, |maybe_handle| {
                    if let Some(handle) = maybe_handle {
                        //TODO set file path text
                        let path = handle.path();
                        let sequence = analysis::parse_file(path);

                        return Message::DataLoaded(sequence);
                    }

                    Message::Done
                })
            }
            Message::DataLoaded(data) => {
                self.chart.set_chart_data(data);

                Task::none()
            }
            Message::NoiseToleranceChanged(value) => {
                self.chart.set_noise_tolerance(value);
                Task::none()
            }
            Message::ChartMessage(msg) => {
                self.chart.update(msg);
                Task::none()
            }
        }
    }
}
