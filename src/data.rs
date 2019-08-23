use nalgebra::geometry::Quaternion;
use std::mem;

#[derive(Debug)]
pub struct Error {
    pub repr: ErrorRepr,
}

#[derive(Debug)]
pub enum ErrorRepr {
    Io(std::io::Error),
    BinCode(bincode::Error),
}

impl From<bincode::Error> for Error {
    #[inline]
    fn from(e: bincode::Error) -> Error {
        Error {
            repr: ErrorRepr::BinCode(e),
        }
    }
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(e: std::io::Error) -> Error {
        Error {
            repr: ErrorRepr::Io(e),
        }
    }
}

#[derive(Debug)]
pub struct Data {
    // unit: microseconds
    pub time: u64,

    // unit: g
    pub accel: [f64; 3],
    // unit: dps
    pub gyro: [f64; 3],
    // unit: uT
    pub mag: [f64; 3],

    // format: ENU
    pub quat: Quaternion<f64>,

    // unit: degrees celsius
    pub temperature: f64,
    // unit: hPa
    pub pressure: f64,
}

impl Default for Data {
    fn default() -> Data {
        Data {
            time: 0,
            accel: [0., 0., 0.],
            gyro: [0., 0., 0.],
            mag: [0., 0., 0.],
            quat: Quaternion::identity(),
            temperature: 0.,
            pressure: 0.,
        }
    }
}

fn read_primitive<S: std::io::Read, R: for<'a> serde::de::Deserialize<'a>>(
    source: &mut S,
) -> Result<R, Error> {
    let mut buffer = vec![0; mem::size_of::<R>()];
    source.read_exact(&mut buffer)?;

    return Ok(bincode::deserialize(&buffer)?);
}

fn filter_eof<R>(r: Result<R, Error>) -> Result<R, Error> {
    match &r {
        Err(e) => match &e.repr {
            ErrorRepr::Io(eio) => match eio.kind() {
                std::io::ErrorKind::UnexpectedEof => Err(Error::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "EOF within sample",
                ))),
                _ => r,
            },
            _ => r,
        },
        Ok(_) => r,
    }
}

pub fn read_sample<S: std::io::Read>(source: &mut S) -> Result<Data, Error> {
    let mut data = Data::default();

    // this is the only one where we let through EOF
    // so the the caller can see the diff between a proper end
    // and an end in the middle of the data
    data.time = read_primitive(source)?;

    for i in 0..data.accel.len() {
        data.accel[i] = filter_eof(read_primitive(source))?;
    }

    for i in 0..data.gyro.len() {
        data.gyro[i] = filter_eof(read_primitive(source))?;
    }

    for i in 0..data.mag.len() {
        data.mag[i] = filter_eof(read_primitive(source))?;
    }

    {
        let _: u64 = filter_eof(read_primitive(source))?;
    }

    data.temperature = filter_eof(read_primitive(source))?;
    data.pressure = filter_eof(read_primitive(source))?;

    {
        let w: f64 = filter_eof(read_primitive(source))?;
        let x: f64 = filter_eof(read_primitive(source))?;
        let y: f64 = filter_eof(read_primitive(source))?;
        let z: f64 = filter_eof(read_primitive(source))?;

        data.quat[0] = x;
        data.quat[1] = y;
        data.quat[2] = z;
        data.quat[3] = w;
    }

    return Ok(data);
}

pub fn read_all_samples<S: std::io::Read>(source: &mut S) -> Result<Vec<Data>, Error> {
    let mut samples = Vec::new();

    loop {
        let sample = match read_sample(source) {
            Err(e) => match &e.repr {
                ErrorRepr::Io(eio) => match eio.kind() {
                    std::io::ErrorKind::UnexpectedEof => break,
                    _ => return Err(e),
                },
                _ => return Err(e),
            },
            Ok(v) => v,
        };

        samples.push(sample);
    }

    return Ok(samples);
}
