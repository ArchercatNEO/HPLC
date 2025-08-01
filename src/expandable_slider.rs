use iced::{
    Element, Length,
    alignment::{Horizontal, Vertical},
    widget::{column, row, slider, text, text_input, toggler},
};

#[derive(Clone, Debug)]
pub enum Message {
    Invalid,
    Value(f32),
    ValueStr(String),
    Start(String),
    End(String),
    Step(String),
    Expanded(bool),
}

#[derive(Clone, Debug)]
pub struct ExpandableSlider {
    value: f32,
    start: f32,
    end: f32,
    step: f32,
    label: String,
    value_str: String,
    start_str: String,
    end_str: String,
    step_str: String,
    expanded: bool,
    exponential: bool,
}

impl ExpandableSlider {
    pub fn new(value: f32, start: f32, end: f32, step: f32, label: &str) -> Self {
        let formatted = format!("{}: ", label);

        Self {
            value,
            start,
            end,
            step,
            label: formatted,
            value_str: value.to_string(),
            start_str: start.to_string(),
            end_str: end.to_string(),
            step_str: step.to_string(),
            expanded: false,
            exponential: false,
        }
    }

    pub fn get_value(&self) -> f32 {
        if self.exponential {
            self.step.powf(self.value)
        } else {
            self.value
        }
    }

    pub fn set_exponential(&mut self, exponential: bool) -> &mut Self {
        self.exponential = exponential;

        self
    }

    pub fn view(&self) -> Element<Message> {
        let number_width = Length::FillPortion(2);

        let expand = toggler(self.expanded).on_toggle(|bit| Message::Expanded(bit));

        let label = text(&self.label)
            .align_x(Horizontal::Right)
            .width(Length::FillPortion(3));

        let bar = slider(self.start..=self.end, self.value, |float| {
            Message::Value(float)
        })
        .step(self.step)
        .width(Length::FillPortion(5));

        if self.expanded {
            let value = {
                let label = if self.exponential {
                    text("base^x = ")
                } else {
                    text("x = ")
                };

                let input = text_input(&self.value_str, &self.value_str)
                    .width(number_width)
                    .on_input(Self::wrap_parse(Message::ValueStr));

                row![label, input].align_y(Vertical::Center)
            };

            let range = {
                let start = text_input(&self.start_str, &self.start_str)
                    .width(number_width)
                    .on_input(Self::wrap_parse(Message::Start));

                let inequality = text("<= x <= ");

                let end = text_input(&self.end_str, &self.end_str)
                    .width(number_width)
                    .on_input(Self::wrap_parse(Message::End));

                row![start, inequality, end]
                    .spacing(5)
                    .align_y(Vertical::Center)
            };

            let step = {
                let label = if self.exponential {
                    text("Base: ")
                } else {
                    text("Step: ")
                };

                let input = text_input(&self.step_str, &self.step_str)
                    .width(number_width)
                    .on_input(Self::wrap_parse(Message::Step));

                row![label, input].align_y(Vertical::Center)
            };

            let top = row![expand, value, range, step].spacing(10);

            let el = column![top, bar];
            el.into()
        } else {
            let info = text(&self.value_str).width(number_width);

            row![expand, label, bar, info].spacing(10).into()
        }
    }

    pub fn update(&mut self, message: Message) -> Option<f32> {
        match message {
            Message::Invalid => None,
            Message::Value(value) => {
                if self.value != value {
                    self.value = value;

                    let transformed = if self.exponential {
                        self.step.powf(self.value)
                    } else {
                        self.value
                    };

                    self.value_str = transformed.to_string();
                    Some(transformed)
                } else {
                    None
                }
            }
            Message::ValueStr(content) => {
                self.value_str = content;

                match self.value_str.parse::<f32>() {
                    Ok(float) => {
                        self.value = float;
                        Some(float)
                    }
                    Err(_) => None,
                }
            }
            Message::Start(content) => {
                if let Ok(float) = content.parse::<f32>() {
                    self.start = float;
                }

                self.start_str = content;
                None
            }
            Message::End(content) => {
                if let Ok(float) = content.parse::<f32>() {
                    self.end = float;
                }

                self.end_str = content;
                None
            }
            Message::Step(content) => {
                if let Ok(float) = content.parse::<f32>() {
                    self.step = float;
                }

                self.step_str = content;
                None
            }
            Message::Expanded(expanded) => {
                self.expanded = expanded;
                None
            }
        }
    }

    fn wrap_parse<F: Fn(String) -> Message>(enum_fn: F) -> impl Fn(String) -> Message {
        move |content: String| {
            let mut numeric = true;
            for character in content.chars() {
                if !character.is_ascii_digit() && character != '.' {
                    numeric = false;
                    break;
                }
            }

            if numeric {
                enum_fn(content)
            } else {
                Message::Invalid
            }
        }
    }
}
