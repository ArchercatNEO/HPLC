use std::path;

use vector::*;

mod app;
mod chromatography;
mod peak;
mod vector;

fn main() -> Result<(), iced::Error> {
    iced::run("HPLC", app::App::update, app::App::view)
}

pub fn parse_file<P: AsRef<path::Path>, U, F: Fn(&str) -> Option<U>>(path: P, fun: F) -> Vec<U> {
    let file = std::fs::read_to_string(path).unwrap();

    let sequence = file
        .split("\n")
        .filter_map(fun);

    sequence.collect()
}

pub fn parse_line_as_lipids(line: &str) -> Option<(f32, String)> {
    let mut data = line.split("\t");

    let x: f32 = {
        let string = data.next();
        if let Some(number) =  string {
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
            name
        } else {
            return None;
        }
    };

    Some((x, lipid.to_string()))
}

pub fn parse_line_as_data(line: &str) -> Option<Point2D> {
    let mut data = line.split("\t");

    let x_str = data.next();
    if x_str == None {
        return None;
    }

    let x_coord = x_str.unwrap().parse();
    if let Err(_) = x_coord {
        return None;
    }

    let x = x_coord.unwrap();

    let y_str = data.next();
    if y_str == None {
        return None;
    }

    let y_coord = y_str.unwrap().parse();
    if let Err(_) = y_coord {
        return None;
    }

    let y = y_coord.unwrap();

    Some(Point2D::new(x, y))
}
