use std::error::Error;

use plotters::prelude::*;

mod peak;

fn main() -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new("quadratic.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let mut chart = ChartBuilder::on(&root)
        .caption("HPLC", ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(5f32..50f32, -2f32..700f32)?;

    chart.configure_mesh().draw()?;

    let data = parse_file();
    
    let sequence = LineSeries::new(data.clone(), &RED);
    chart
        .draw_series(sequence)?
        .label("data")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    
    let peaks: Vec<(f32, f32)> = peak::Peak::find_peaks(data)
        .iter().flat_map(|peak| {
            [peak.start, peak.turning_point, peak.end]
        }).collect();
    
    let peak_points: PointSeries<'_, (f32, f32), Vec<(f32, f32)>, Circle<(f32, f32), i32>, i32> = PointSeries::new(peaks, 1, &BLUE);
    
    chart
        .draw_series(peak_points)?
        .label("turning points")
        .legend(|(x, y)| Circle::new((x, y), 1, &BLUE));


    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}

fn parse_file<'a>() -> Vec<(f32, f32)> {
    let file = std::fs::read_to_string("./src/data.tsv").unwrap();

    let sequence = file
        .split("\n")
        .filter(|line| !line.is_empty())
        .filter_map(parse_line)
        .filter(|coord| {
            5.0 < coord.0 && coord.0 < 50.0
        });

    sequence.collect()
}

fn parse_line(line: &str) -> Option<(f32, f32)> {
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


    Some((x, y))
}