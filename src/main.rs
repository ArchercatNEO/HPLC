mod app;
mod chromatogram;
mod chromatography;
mod cubic;
mod expandable_slider;
mod peak;
mod reference;
mod vector;

fn main() -> Result<(), iced::Error> {
    iced::run("HPLC", app::App::update, app::App::view)
}
