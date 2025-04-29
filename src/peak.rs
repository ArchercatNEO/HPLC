#[derive(Default)]
pub struct Peak {
    pub start: (f32, f32),
    pub turning_point: (f32, f32),
    pub end: (f32, f32)
}

impl Peak {
    pub fn find_peaks<T: IntoIterator<Item = (f32, f32)>>(iter: T) -> Vec<Self> {
        let mut result = vec![];

        let mut peak = Peak::default();
        for point in iter {
            if peak.start == (0.0, 0.0) {
                peak.start = point;
                continue;
            }

            if peak.turning_point.1 < point.1 {
                peak.turning_point = point;
                continue;
            }

            if peak.end == (0.0, 0.0) || peak.end.1 > point.1 {
                peak.end = point;
            } else {
                if peak.turning_point.1 - peak.start.1 > 50.0 {
                    result.push(peak);
                } 
                peak = Peak::default();
            }
        }

        result
    }

    // area of trapezium = 1/2 * (a + b) * h
    pub fn area(&self, start_basline: f32, end_baseline: f32) -> f32 {
        let midpoint_baseline = (start_basline + end_baseline) / 2.0;

        let left = {
            let height = self.turning_point.0 - self.start.0;
            let a = self.start.1 - start_basline;
            let b = self.turning_point.1 - midpoint_baseline;
            0.5 * (a + b) * height
        };

        let right = {
            let height = self.end.0 - self.turning_point.0;
            let a = self.turning_point.1 - midpoint_baseline;
            let b = self.start.1 - end_baseline;
            0.5 * (a + b) * height
        };

        left + right
    }
}