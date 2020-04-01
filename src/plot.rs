use crate::*;

pub struct Plot(Python);

pub const COLOR_A: &str = "#1f77b4";
pub const COLOR_M: &str = "#ff7f0e";
pub const COLOR_E: &str = "#2ca02c";

impl Plot {
    pub fn new<T: serde::ser::Serialize>(code: &T) -> Result<Plot, Error> {
        let mut plot = Self(Python::new(
            &"\
            import numpy as np\n\
            import matplotlib.pyplot as plt\n\
            exec(load_data())\n\
        ",
        )?);
        plot.0.write(code)?;
        Ok(plot)
    }

    pub fn write<T: serde::ser::Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.0.write(value)
    }

    pub fn wait(&mut self) -> Result<(), Error> {
        self.0.wait()
    }
}

pub struct TimeDataPlot(Plot);

impl TimeDataPlot {
    pub fn new<T: serde::ser::Serialize>(nplots: usize, time: &T) -> Result<Self, Error> {
        let mut plot = Self(Plot::new(&include_str!("python/plot_time_data.py"))?);
        plot.0.write(&nplots)?;
        plot.0.write(time)?;
        Ok(plot)
    }

    pub fn set_title(&mut self, id: usize, title: &str) -> Result<(), Error> {
        self.0.write(&"titl")?;
        self.0.write(&id)?;
        self.0.write(&title)?;
        Ok(())
    }

    pub fn plot<D: serde::ser::Serialize>(
        &mut self,
        id: usize,
        color: &str,
        data: &D,
    ) -> Result<(), Error> {
        self.0.write(&"plot")?;
        self.0.write(&id)?;
        self.0.write(&color)?;
        self.0.write(data)?;
        Ok(())
    }

    pub fn plot_sample<'a, F, ST>(
        &mut self,
        id: usize,
        color: &str,
        samples: &'a [Data],
        f: F,
    ) -> Result<(), Error>
    where
        F: Fn(&'a Data) -> ST,
        ST: 'a + serde::ser::Serialize,
    {
        self.plot(id, color, &DataSerializer::new(samples, |_i, v| f(v)))
    }

    pub fn plot_actual<'a, F, ST>(
        &mut self,
        id: usize,
        color: &str,
        samples: &'a [Data],
        f: F,
    ) -> Result<(), Error>
    where
        F: Fn(&'a ndarray::Array1<f64>) -> ST,
        ST: 'a + serde::ser::Serialize,
    {
        let has_actual = match samples.first() {
            Some(sample) => sample.actual.is_some(),
            None => false,
        };

        if !has_actual {
            return Ok(());
        }

        self.plot(
            id,
            color,
            &DataSerializer::new(samples, |_i, v| f(v.actual.as_ref().unwrap())),
        )
    }

    pub fn plot_array<'a, S, A, F, ST>(
        &mut self,
        id: usize,
        color: &str,
        arr: &'a [ndarray::ArrayBase<S, ndarray::Ix1>],
        f: F,
    ) -> Result<(), Error>
    where
        S: ndarray::Data<Elem = A>,
        F: Fn(&'a ndarray::ArrayBase<S, ndarray::Ix1>) -> ST,
        ST: 'a + serde::ser::Serialize,
    {
        self.plot(id, color, &DataSerializer::new(arr, |_i, v| f(v)))
    }

    pub fn show(&mut self) -> Result<(), Error> {
        let v: Option<&str> = None;
        self.0.write(&v)?;
        self.0.wait()
    }
}

pub struct AMEPlot<'a, 'b> {
    plot: TimeDataPlot,
    samples: &'a [Data],
    est: &'b [ndarray::Array1<f64>],
    id: usize,
}

impl<'a, 'b> AMEPlot<'a, 'b> {
    pub fn new(
        nplots: usize,
        samples: &'a [Data],
        est: &'b [ndarray::Array1<f64>],
    ) -> Result<Self, Error> {
        let plot = TimeDataPlot::new(
            nplots,
            &DataSerializer::new(&samples, |_i, v| v.time_seconds()),
        )?;
        Ok(Self {
            plot,
            samples,
            est,
            id: 0,
        })
    }

    pub fn add<FA, FB, STA, STB>(
        &mut self,
        title: &str,
        f_sample: FB,
        f_state: FA,
    ) -> Result<(), Error>
    where
        FA: Fn(&ndarray::Array1<f64>) -> STA,
        FB: Fn(&Data) -> STB,
        STA: serde::ser::Serialize,
        STB: serde::ser::Serialize,
    {
        self.plot.set_title(self.id, title)?;
        self.plot
            .plot_actual(self.id, COLOR_A, self.samples, &f_state)?;
        self.plot
            .plot_sample(self.id, COLOR_M, self.samples, &f_sample)?;
        self.plot.plot_array(self.id, COLOR_E, self.est, &f_state)?;

        self.id += 1;

        Ok(())
    }

    pub fn add_nm<FA, STA>(&mut self, title: &str, f_state: FA) -> Result<(), Error>
    where
        FA: Fn(&ndarray::Array1<f64>) -> STA,
        STA: serde::ser::Serialize,
    {
        self.plot.set_title(self.id, title)?;
        self.plot
            .plot_actual(self.id, COLOR_A, self.samples, &f_state)?;
        self.plot.plot_array(self.id, COLOR_E, self.est, &f_state)?;

        self.id += 1;

        Ok(())
    }

    pub fn show(&mut self) -> Result<(), Error> {
        self.plot.show()
    }
}
