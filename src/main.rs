mod app;
mod chromatogram;
mod chromatography;
mod expandable_slider;
mod peak;
mod reference;
mod spline;
mod vector;

fn main() -> iced::Result {
    iced::application("HPLC", app::App::update, app::App::view)
        .theme(|_| iced::theme::Theme::Light)
        .run()
}
