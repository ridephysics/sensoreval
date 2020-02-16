use crate::*;

pub struct Plot {
    child: std::process::Child,
}

impl Plot {
    pub fn new(code: &str) -> Result<Plot, Error> {
        let mut child = std::process::Command::new("python")
            .args(&[
                "-c",
                "\
                 import sys\n\
                 import pickle\n\
                 import numpy as np\n\
                 import matplotlib.pyplot as plt\n\
                 def load_data():\n\
                     \treturn pickle.load(sys.stdin.buffer)\n\
                 exec(load_data())\n\
                 ",
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::inherit())
            .spawn()?;

        let stdin = child.stdin.as_mut().unwrap();
        serde_pickle::to_writer(stdin, &code, true)?;

        Ok(Plot { child })
    }

    pub fn write<T: serde::ser::Serialize>(&mut self, value: &T) -> Result<(), Error> {
        let stdin = self.child.stdin.as_mut().unwrap();
        serde_pickle::to_writer(stdin, value, true)?;

        Ok(())
    }

    pub fn wait(&mut self) -> Result<(), Error> {
        let status = self.child.wait()?;
        if !status.success() {
            return Err(Error::from(status));
        }

        Ok(())
    }
}

pub struct TimeDataPlot(Plot);

impl TimeDataPlot {
    pub fn new<T: serde::ser::Serialize>(time: &T) -> Result<TimeDataPlot, Error> {
        let mut plot = Self(Plot::new(include_str!("python/plot_time_data.py"))?);
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
