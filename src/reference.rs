use std::fs;
use std::path;

use crate::spline::Spline;

#[derive(Clone, Debug, PartialEq)]
enum ExpectedLocation {
    RetentionTime(f64),
    GlucoseUnit(f64),
    Complete(f64, f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reference {
    pub name: Option<String>,
    expected_location: ExpectedLocation,
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

                type ReferenceFn = &'static dyn Fn(&mut ReferenceBuilder, &str);

                let funcs: Vec<ReferenceFn> = entries
                    .map(|entry| {
                        if entry == "Name" {
                            &ReferenceBuilder::parse_name
                        } else if entry == "RT" {
                            &ReferenceBuilder::parse_retention_time
                        } else if entry == "GU" {
                            &ReferenceBuilder::parse_glucose_units
                        } else {
                            let func: ReferenceFn = &ReferenceBuilder::parse_none;
                            func
                        }
                    })
                    .collect();

                lines
                    .filter_map(|line| {
                        let mut reference_builder = ReferenceBuilder::default();

                        let entries = if line.contains("\t") {
                            line.split("\t")
                        } else {
                            line.split(",")
                        };

                        for (entry, func) in entries.zip(&funcs) {
                            func(&mut reference_builder, entry);
                        }

                        match reference_builder.location {
                            None => {
                                match &reference_builder.name {
                                    Some(name) => eprintln!(
                                        "Lipid {} needs at least one of retention time or GU",
                                        name
                                    ),
                                    None => eprintln!(
                                        "Unnamed lipid needs at least one of retention time or GU"
                                    ),
                                }

                                None
                            }
                            Some(location) => {
                                let reference = Reference {
                                    name: reference_builder.name,
                                    expected_location: location,
                                };

                                Some(reference)
                            }
                        }
                    })
                    .collect()
            }
            Err(err) => {
                println!("{}", err);
                vec![]
            }
        }
    }

    pub fn get_expected_location(&self, spline: Option<&Spline>) -> Option<f64> {
        match (&self.expected_location, spline) {
            (ExpectedLocation::RetentionTime(rt), None) => Some(*rt),
            (ExpectedLocation::RetentionTime(rt), Some(spline)) => spline.evaluate(*rt),
            (ExpectedLocation::GlucoseUnit(_), None) => None,
            (ExpectedLocation::GlucoseUnit(gu), Some(_)) => Some(*gu),
            (ExpectedLocation::Complete(rt, _), None) => Some(*rt),
            (ExpectedLocation::Complete(_, gu), Some(_)) => Some(*gu),
        }
    }

    pub fn get_expected_rt(&self) -> Option<f64> {
        match &self.expected_location {
            ExpectedLocation::RetentionTime(rt) => Some(*rt),
            ExpectedLocation::GlucoseUnit(_) => None,
            ExpectedLocation::Complete(rt, _) => Some(*rt),
        }
    }

    pub fn get_expected_gu(&self, spline: Option<&Spline>) -> Option<f64> {
        match (&self.expected_location, spline) {
            (ExpectedLocation::RetentionTime(_), None) => None,
            (ExpectedLocation::RetentionTime(rt), Some(spline)) => spline.evaluate(*rt),
            (ExpectedLocation::GlucoseUnit(gu), _) => Some(*gu),
            (ExpectedLocation::Complete(_, gu), _) => Some(*gu),
        }
    }
}

#[derive(Default)]
struct ReferenceBuilder {
    pub name: Option<String>,
    pub location: Option<ExpectedLocation>,
}

impl ReferenceBuilder {
    fn parse_none(&mut self, _: &str) {}

    fn parse_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }

    fn parse_retention_time(&mut self, rt: &str) {
        match rt.parse::<f64>() {
            Ok(rt) => match self.location {
                None => {
                    self.location = Some(ExpectedLocation::RetentionTime(rt));
                }
                Some(ExpectedLocation::GlucoseUnit(gu)) => {
                    self.location = Some(ExpectedLocation::Complete(rt, gu));
                }
                Some(_) => {}
            },
            Err(_) => {}
        }
    }

    fn parse_glucose_units(&mut self, gu: &str) {
        match gu.parse::<f64>() {
            Err(_) => {}
            Ok(gu) => match self.location {
                None => {
                    self.location = Some(ExpectedLocation::GlucoseUnit(gu));
                }
                Some(ExpectedLocation::RetentionTime(rt)) => {
                    self.location = Some(ExpectedLocation::Complete(rt, gu));
                }
                Some(_) => {}
            },
        }
    }
}
