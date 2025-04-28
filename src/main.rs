use std::error::Error;

use plotters::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new("quadratic.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let mut chart = ChartBuilder::on(&root)
        .caption("y=x^2", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(5f32..50f32, -1f32..700f32)?;

    chart.configure_mesh().draw()?;

    let sequence = parse_file();

    chart
        .draw_series(sequence)?
        .label("y = x^2")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}

fn parse_file<'a>() -> LineSeries<BitMapBackend<'a>, (f32, f32)> {
    let file = std::fs::read_to_string("./src/CSF1.arw").unwrap();

    let sequence = file
        .split("\n")
        .filter(|line| !line.is_empty())
        .map(parse_line)
        .filter_map(|x| x)
        .filter(|coord| {
            5.0 < coord.0 && coord.0 < 50.0
        });
        

    LineSeries::new(sequence, &RED)
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