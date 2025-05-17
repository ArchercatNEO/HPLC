use std::path;

use vector::*;

mod app;
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
    let file = std::fs::read_to_string(path).unwrap();

    println!("{}", file);

    let sequence = file.lines().filter_map(fun);

    sequence.collect()
}

pub fn parse_line_as_lipids(line: &str) -> Option<(f32, String)> {
    println!("{}", line);
    
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
    println!("{}", line);

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
