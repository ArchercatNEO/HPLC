mod app;
mod chromatogram;
mod chromatography;
mod expandable_slider;
mod exporter;
mod peak;
mod reference;
mod spline;
mod vector;

use iced::Theme;

use crate::app::App;

fn main() -> iced::Result {
    iced::daemon("HPLC", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_, _| Theme::Light)
        .run_with(App::new)
}
