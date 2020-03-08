use crate::*;

pub struct Python {
    child: std::process::Child,
}

impl Python {
    pub fn new_args<I, Sp, Sa, T>(program: Sp, args: I, code: &T) -> Result<Self, Error>
    where
        I: IntoIterator<Item = Sa>,
        Sa: AsRef<std::ffi::OsStr>,
        Sp: AsRef<std::ffi::OsStr>,
        T: serde::ser::Serialize,
    {
        let mut child = std::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::inherit())
            .spawn()?;

        let stdin = child.stdin.as_mut().unwrap();
        serde_pickle::to_writer(stdin, &code, true)?;

        Ok(Self { child })
    }

    pub fn new<T: serde::ser::Serialize>(code: &T) -> Result<Self, Error> {
        Self::new_args(
            "python",
            &[
                "-c",
                "\
                 import sys\n\
                 import pickle\n\
                 def load_data():\n\
                     \treturn pickle.load(sys.stdin.buffer)\n\
                 exec(load_data())\n\
                 ",
            ],
            code,
        )
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
