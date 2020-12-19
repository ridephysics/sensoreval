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

        Ok(())
    }
}
