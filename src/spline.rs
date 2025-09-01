use std::{fmt::Debug, ops::RangeInclusive};

use crate::vector::{Point2D, Vector2};

#[derive(Clone, Debug)]
pub struct Spline {
    cubics: Vec<(RangeInclusive<f64>, Cubic)>,
}

impl Spline {
    pub fn new(points: &[Point2D]) -> Option<Self> {
        let order = (points.len() - 1) * 4;
        let mut matrix: Vec<Vec<f64>> = Vec::with_capacity(order);
        let mut result = Vec::with_capacity(order);

        for (i, point) in points.iter().enumerate() {
            // First and last points are special
            if i == 0 {
                // Second derivative == 0
                let mut startpoint = vec![0.0; order];
                startpoint[0] = 6.0 * point.x();
                startpoint[1] = 2.0;

                matrix.push(startpoint);
                result.push(0.0);

                let mut equation = vec![0.0; order];
                equation[0] = point.x().powi(3);
                equation[1] = point.x().powi(2);
                equation[2] = point.x();
                equation[3] = 1.0;

                matrix.push(equation);
                result.push(point.y());

                continue;
            }

            if i == points.len() - 1 {
                // Second derivative == 0
                let mut endpoint = vec![0.0; order];
                endpoint[i * 4 - 4] = 6.0 * point.x();
                endpoint[i * 4 - 3] = 2.0;

                matrix.push(endpoint);
                result.push(0.0);

                let mut equation = vec![0.0; order];
                equation[i * 4 - 4] = point.x().powi(3);
                equation[i * 4 - 3] = point.x().powi(2);
                equation[i * 4 - 2] = point.x();
                equation[i * 4 - 1] = 1.0;

                matrix.push(equation);
                result.push(point.y());

                continue;
            }

            // ax^3 + bx^2 + cx + d = y
            let mut prev_equation = vec![0.0; order];
            prev_equation[i * 4 - 4] = point.x().powi(3);
            prev_equation[i * 4 - 3] = point.x().powi(2);
            prev_equation[i * 4 - 2] = point.x();
            prev_equation[i * 4 - 1] = 1.0;

            matrix.push(prev_equation);
            result.push(point.y());

            // ax^3 + bx^2 + cx + d = y
            let mut next_equation = vec![0.0; order];
            next_equation[i * 4] = point.x().powi(3);
            next_equation[i * 4 + 1] = point.x().powi(2);
            next_equation[i * 4 + 2] = point.x();
            next_equation[i * 4 + 3] = 1.0;

            matrix.push(next_equation);
            result.push(point.y());

            // 3ax^2 + 2bx + c
            // -3ax^2 - 2bx - c
            // = 0
            let mut derivative = vec![0.0; order];
            derivative[i * 4 - 4] = 3.0 * point.x().powi(2);
            derivative[i * 4 - 3] = 2.0 * point.x();
            derivative[i * 4 - 2] = 1.0;

            derivative[i * 4] = -3.0 * point.x().powi(2);
            derivative[i * 4 + 1] = -2.0 * point.x();
            derivative[i * 4 + 2] = -1.0;

            matrix.push(derivative);
            result.push(0.0);

            let mut inflection = vec![0.0; order];
            inflection[i * 4 - 4] = 6.0 * point.x();
            inflection[i * 4 - 3] = 2.0;

            inflection[i * 4] = -6.0 * point.x();
            inflection[i * 4 + 1] = -2.0;

            matrix.push(inflection);
            result.push(0.0);
        }

        let maybe_splines = Self::solve_matrix(&mut matrix, &mut result);
        if let Some(splines) = maybe_splines {
            let (iter, _) = splines.as_chunks::<4>();
            let mut cubics = Vec::with_capacity(iter.len());

            for (i, coefficients) in iter.iter().enumerate() {
                let start = points[i].x();
                let end = points[i + 1].x();
                let cubic = Cubic::new(
                    coefficients[0],
                    coefficients[1],
                    coefficients[2],
                    coefficients[3],
                );
                cubics.push((start..=end, cubic));
            }

            return Some(Spline { cubics });
        }

        None
    }

    pub fn evaluate(&self, value: f64) -> Option<f64> {
        for (range, cubic) in &self.cubics {
            if range.contains(&value) {
                return Some(cubic.evaluate(value));
            }
        }

        return None;
    }

    fn solve_matrix(matrix: &mut [Vec<f64>], values: &mut [f64]) -> Option<Vec<f64>> {
        let order = matrix.len();

        for i in 0..order {
            // Partial pivoting
            let mut max_row = i;
            for k in (i + 1)..order {
                if matrix[k][i].abs() > matrix[max_row][i].abs() {
                    max_row = k;
                }
            }

            // Swap rows in matrix and vector
            matrix.swap(i, max_row);
            values.swap(i, max_row);

            // Check for singular matrix
            if matrix[i][i].abs() < 1e-12 {
                return None; // Singular or nearly singular matrix
            }

            // Eliminate entries below pivot
            for k in (i + 1)..order {
                let factor = matrix[k][i] / matrix[i][i];
                for j in i..order {
                    matrix[k][j] -= factor * matrix[i][j];
                }
                values[k] -= factor * values[i];
            }
        }

        // Back substitution
        let mut x = vec![0.0; order];
        for i in (0..order).rev() {
            let mut sum = values[i];
            for j in (i + 1)..order {
                sum -= matrix[i][j] * x[j];
            }
            x[i] = sum / matrix[i][i];
        }

        Some(x)
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Cubic {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

impl Cubic {
    pub fn new(a: f64, b: f64, c: f64, d: f64) -> Self {
        Self { a, b, c, d }
    }

    pub fn evaluate(&self, x: f64) -> f64 {
        self.a * x * x * x + self.b * x * x + self.c * x + self.d
    }
}

impl Debug for Cubic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formula = format!("{}x^3 + {}x^2 + {}x + {}", self.a, self.b, self.c, self.d);
        f.write_str(&formula)
    }
}

impl Default for Cubic {
    fn default() -> Self {
        Self {
            a: 0.0,
            b: 0.0,
            c: 1.0,
            d: 0.0,
        }
    }
}
