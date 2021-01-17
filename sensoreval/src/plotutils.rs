use crate::Data;
use crate::Error;
use ndarray_linalg::Norm;

pub trait PlotUtils {
    fn add_measurements(&mut self, samples: &[Data], x: &[f64]) -> Result<(), Error>;
}

impl<'a> PlotUtils for sensoreval_utils::Plot<'a> {
    fn add_measurements(&mut self, samples: &[Data], x: &[f64]) -> Result<(), Error> {
        let mut t = Self::default_line();
        t.line().color(sensoreval_utils::COLOR_M);
        t.name("measurement");
        t.x(&x);

        for i in 0..3 {
            let y: Vec<f64> = samples.iter().map(|s| s.accel[i]).collect();
            self.add_trace_to_rowname_ensure(
                &mut t.clone().y(&y),
                Self::axisid_to_rowname("acc", i),
            )?;
        }

        for i in 0..3 {
            let y: Vec<f64> = samples.iter().map(|s| s.gyro[i]).collect();
            self.add_trace_to_rowname_ensure(
                &mut t.clone().y(&y),
                Self::axisid_to_rowname("gyr", i),
            )?;
        }

        for i in 0..3 {
            let y: Vec<f64> = samples.iter().map(|s| s.mag[i]).collect();
            self.add_trace_to_rowname_ensure(
                &mut t.clone().y(&y),
                Self::axisid_to_rowname("mag", i),
            )?;
        }

        {
            let y: Vec<f64> = samples.iter().map(|s| s.accel.norm_l2()).collect();
            self.add_trace_to_rowname_ensure(&mut t.clone().y(&y), "norm-a")?;
        }

        {
            let y: Vec<f64> = samples
                .iter()
                .map(|s| s.gyro.norm_l2().copysign(s.gyro[0]))
                .collect();
            self.add_trace_to_rowname_ensure(&mut t.clone().y(&y), "norm-g")?;
        }

        {
            let y: Vec<f64> = samples.iter().map(|s| s.mag.norm_l2()).collect();
            self.add_trace_to_rowname_ensure(&mut t.clone().y(&y), "mag-g")?;
        }

        {
            let y: Vec<f64> = samples.iter().map(|s| s.pressure).collect();
            self.add_trace_to_rowname_ensure(&mut t.clone().y(&y), "baro")?;
        }

        {
            let y: Vec<f64> = samples.iter().map(|s| s.temperature).collect();
            self.add_trace_to_rowname_ensure(&mut t.clone().y(&y), "temp")?;
        }

        let (has_actual, actual_len) = match samples.first() {
            Some(sample) => match sample.actual.as_ref() {
                Some(actual) => (true, actual.len()),
                None => (false, 0),
            },
            None => (false, 0),
        };
        if has_actual {
            t.name("actual");
            t.line().color(sensoreval_utils::COLOR_A);

            for i in 0..actual_len {
                let y: Vec<f64> = samples
                    .iter()
                    .map(|s| s.actual.as_ref().unwrap()[i])
                    .collect();

                let mut trace = Self::default_line();
                trace.x(&x).y(&y).name("actual");
                trace.line().color(sensoreval_utils::COLOR_A);

                let rowid = self.ensure_row(format!("x{}", i))?;
                self.add_trace_to_rowid(&mut trace, rowid)?;
            }
        }

        Ok(())
    }
}
