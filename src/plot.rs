use crate::*;

pub struct Plot(Python);

impl Plot {
    pub fn new<T: serde::ser::Serialize>(code: &T) -> Result<Plot, Error> {
        let mut plot = Self(Python::new(
            &"\
            import numpy as np\n\
            import matplotlib.pyplot as plt\n\
            ca = '#1f77b4'\n\
            cz = '#ff7f0e'\n\
            ce = '#2ca02c'\n\
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
    pub fn new<T: serde::ser::Serialize>(time: &T) -> Result<TimeDataPlot, Error> {
        let mut plot = Self(Plot::new(&include_str!("python/plot_time_data.py"))?);
        plot.0.write(time)?;
        Ok(plot)
    }

    pub fn add<D: serde::ser::Serialize>(&mut self, data: &D) -> Result<(), Error> {
        let isdata: bool = true;

        self.0.write(&isdata)?;
        self.0.write(&data)?;

        Ok(())
    }

    pub fn show(&mut self) -> Result<(), Error> {
        let isdata: bool = false;
        self.0.write(&isdata)?;

        self.0.wait()
    }
}
