use std::fs;
use std::path;

#[derive(Clone, Debug, Default)]
pub struct Reference {
    pub name: String,
    pub retention_time: f32,
    pub glucose_units: f32,
}

impl Reference {
    pub fn parse_file<P: AsRef<path::Path>>(path: &P) -> Vec<Self> {
        let file = fs::read_to_string(path);
        match file {
            Ok(content) => {
                let mut lines = content.lines();
                let header = lines.next().unwrap();

                let entries = if header.contains("\t") {
                    header.split("\t")
                } else {
                    header.split(",")
                };

                let mut funcs: Vec<&dyn Fn(Reference, &str) -> Reference> = Vec::with_capacity(4);

                for entry in entries {
                    if entry == "name" {
                        funcs.push(&Reference::parse_name);
                    } else if entry == "retention" {
                        funcs.push(&Reference::parse_retention_time);
                    } else if entry == "GU" {
                        funcs.push(&Reference::parse_glucose_units);
                    } else {
                        funcs.push(&Reference::parse_none);
                    }
                }

                lines
                    .map(|line| {
                        let mut reference = Reference::default();

                        let entries = if line.contains("\t") {
                            line.split("\t")
                        } else {
                            line.split(",")
                        };

                        for (entry, func) in entries.zip(&funcs) {
                            reference = func(reference, entry);
                        }

                        reference
                    })
                    .collect()
            }
            Err(err) => {
                println!("{}", err);
                vec![]
            }
        }
    }

    pub fn parse_none(self, _none: &str) -> Self {
        self
    }

    pub fn parse_name(mut self, name: &str) -> Self {
        self.name = name.trim().to_string();

        self
    }

    pub fn parse_retention_time(mut self, retention_time: &str) -> Self {
        if let Ok(time) = retention_time.parse::<f32>() {
            self.retention_time = time;
        }

        self
    }

    pub fn parse_glucose_units(mut self, glucose_units: &str) -> Self {
        if let Ok(gu) = glucose_units.parse::<f32>() {
            self.glucose_units = gu;
        }

        self
    }
}
