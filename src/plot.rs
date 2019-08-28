use crate::*;

pub struct Plot {
    child: std::process::Child,
}

impl Plot {
    pub fn new<T: serde::ser::Serialize>(time: &T) -> Result<Plot, Error> {
        let code = include_str!("python/plot_time_data.py");

        let mut child = std::process::Command::new("python")
            .args(&[
                "-c",
                "import sys\nimport pickle\nexec(pickle.load(sys.stdin.buffer))",
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::inherit())
            .spawn()?;

        let stdin = child.stdin.as_mut().unwrap();
        serde_pickle::to_writer(stdin, &code, true)?;
        serde_pickle::to_writer(stdin, &time, true)?;

        return Ok(Plot { child: child });
    }

    pub fn add<D: serde::ser::Serialize>(&mut self, data: &D) -> Result<(), Error> {
        let isdata: bool = true;
        let stdin = self.child.stdin.as_mut().unwrap();

        serde_pickle::to_writer(stdin, &isdata, true)?;
        serde_pickle::to_writer(stdin, &data, true)?;

        return Ok(());
    }

    pub fn show(&mut self) -> Result<(), Error> {
        let isdata: bool = false;
        let stdin = self.child.stdin.as_mut().unwrap();

        serde_pickle::to_writer(stdin, &isdata, true)?;

        let status = self.child.wait()?;
        if !status.success() {
            return Err(Error::from(status));
        }

        return Ok(());
    }
}
