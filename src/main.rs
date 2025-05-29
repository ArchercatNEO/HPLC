use std::{fs, path};

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

pub fn parse_file<P: AsRef<path::Path>, U, F: Fn(&str) -> Option<U>>(path: &P, fun: F) -> Vec<U> {
    let file = fs::read_to_string(path);
    match file {
        Ok(content) => content.lines().filter_map(fun).collect(),
        Err(err) => {
            println!("{}", err);
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

    data.next();

    let lipid = {
        let string = data.next();
        if let Some(name) = string {
            name.trim()
        } else {
            return None;
        }
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

pub fn parse_header<P: AsRef<path::Path>>(path: &P) -> Option<String> {
    let file = fs::read_to_string(path);
    file.map_or(None, |content| {
        let header = content.lines().next().unwrap_or("");
        let mut data = header.split("\t");
        data.next();
        data.next().map(|slice| slice.to_string())
    })
}
