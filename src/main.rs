use std::{io::Read, os::unix::fs::MetadataExt, path};

use vector::*;

mod app;
mod chromatogram;
mod chromatography;
mod peak;
mod vector;

fn main() -> Result<(), iced::Error> {
    iced::run("HPLC", app::App::update, app::App::view)
}

pub fn round_to_precision(value: f32, decimals: i32) -> f32 {
    let power = 10.0f32.powi(decimals);
    (value * power).round() / power
}

pub fn parse_file<P: AsRef<path::Path>, U, F: Fn(&str) -> Option<U>>(path: P, fun: F) -> Vec<U> {
    let mut file = std::fs::File::open(path).unwrap();
    let size = file.metadata().unwrap().size();
    let mut buffer: Vec<u8> = vec![0; size.try_into().unwrap()];
    let result = file.read(&mut buffer);

    match result {
        Ok(_) => {
            let content = String::from_utf8(buffer);
            let ret = match content {
                Ok(data) => {
                    println!("file content {}", data);
                    data.lines().filter_map(fun).collect()
                }
                Err(err) => {
                    println!("convert to string err {}", err);
                    vec![]
                }
            };
            println!("read {} bytes", size);
            ret
        }
        Err(err) => {
            println!("failed {}", err);
            vec![]
        }
    }
}

pub fn parse_line_as_lipids(line: &str) -> Option<(f32, String)> {
    let mut data = if line.contains("\t") {
        line.split("\t")
    } else {
        line.split(",")
    };

    let x: f32 = {
        let string = data.next();
        if let Some(number) = string {
            if let Ok(float) = number.parse() {
                float
            } else {
                return None;
            }
        } else {
            return None;
        }
    };

    let lipid = {
        let string = data.next();
        if let Some(name) = string {
            name.trim()
        } else {
            return None;
        }
    };

    Some((x, lipid.to_string()))
}

pub fn parse_line_as_data(line: &str) -> Option<Point2D> {
    let mut data = if line.contains("\t") {
        line.split("\t")
    } else {
        line.split(",")
    };

    let x: f32 = {
        let string = data.next();
        if let Some(number) = string {
            if let Ok(float) = number.parse() {
                float
            } else {
                return None;
            }
        } else {
            return None;
        }
    };

    let y: f32 = {
        let string = data.next();
        if let Some(number) = string {
            if let Ok(float) = number.parse() {
                float
            } else {
                return None;
            }
        } else {
            return None;
        }
    };

    Some(Point2D::new(x, y))
}
