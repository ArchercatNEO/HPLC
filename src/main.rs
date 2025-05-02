mod analysis;
mod app;
mod chart;

fn main() -> Result<(), iced::Error> {
    app::App::start()
}
