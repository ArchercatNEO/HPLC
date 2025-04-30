use crate::chart::HplcChart;

#[derive(Debug, Default)]
pub struct App;

impl App {
    pub fn start() -> iced::Result {
        iced::run("HPLC", HplcChart::update, HplcChart::view)
    }
}
