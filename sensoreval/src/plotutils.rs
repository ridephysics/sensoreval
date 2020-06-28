use crate::Data;
use crate::Error;

pub trait PlotUtils {
    fn add_measurements(&mut self, samples: &[Data], x: &[f64]) -> Result<(), Error>;
}

impl<'a> PlotUtils for sensoreval_utils::Plot<'a> {
    fn add_measurements(&mut self, samples: &[Data], x: &[f64]) -> Result<(), Error> {
        let mut add_row = |rowname: &str, id: usize, y: &[f64]| -> Result<(), Error> {
            let mut t = Self::default_line();
            t.x(&x).y(&y).name("measurement");
            t.line().color(sensoreval_utils::COLOR_M);

            let rowid = self.ensure_row(Self::axisid_to_rowname(rowname, id))?;
            self.add_trace_to_rowid(&mut t, rowid)?;

            Ok(())
        };

        for i in 0..3 {
            let y: Vec<f64> = samples.iter().map(|s| s.accel[i]).collect();
            add_row("acc", i, &y)?;
        }

        for i in 0..3 {
            let y: Vec<f64> = samples.iter().map(|s| s.gyro[i]).collect();
            add_row("gyr", i, &y)?;
        }

        for i in 0..3 {
            let y: Vec<f64> = samples.iter().map(|s| s.mag[i]).collect();
            add_row("mag", i, &y)?;
        }

        Ok(())
    }
}
