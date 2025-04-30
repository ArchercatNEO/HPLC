mod app;
mod chart;
mod peak;

fn main() -> Result<(), iced::Error> {
    app::App::start()
}
