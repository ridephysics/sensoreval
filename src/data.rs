use crate::error::*;

use nalgebra::base::Vector3;
use nalgebra::geometry::{Quaternion, UnitQuaternion};
use serde::ser::{Serialize, SerializeSeq, Serializer};

#[derive(Debug)]
pub struct Data {
    // unit: microseconds
    pub time: u64,

    // unit: g
    pub accel: Vector3<f64>,
    // unit: dps
    pub gyro: Vector3<f64>,
    // unit: uT
    pub mag: Vector3<f64>,

    // format: ENU
    pub quat: UnitQuaternion<f64>,

    // unit: degrees celsius
    pub temperature: f64,
    // unit: hPa
    pub pressure: f64,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            time: 0,
            accel: Vector3::new(0., 0., 0.),
            gyro: Vector3::new(0., 0., 0.),
            mag: Vector3::new(0., 0., 0.),
            quat: UnitQuaternion::identity(),
            temperature: 0.,
            pressure: 0.,
        }
    }
}

impl Data {
    fn pressure_altitude_feet(&self) -> f64 {
        return 145366.45 * (1.0 - (self.pressure/1013.25).powf(0.190284));
    }

    pub fn pressure_altitude(&self) -> f64 {
        return self.pressure_altitude_feet() * 0.3048;
    }
}

macro_rules! create_serializer(
    ($type:ident,
     $var:ident,
     $name:ident,
     $value:expr) => {
        pub struct $name<'a>(&'a Vec<$type>);

        impl<'a> Serialize for $name<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
                for $var in self.0 {
                    seq.serialize_element($value)?;
                }
                seq.end()
            }
        }

        impl<'a> From<&'a Vec<$type>> for $name<'a> {
            fn from(dataset: &'a Vec<$type>) -> $name<'a> {
                $name(dataset)
            }
        }
    }
);

create_serializer!(
    Data, data,
    AccelDataSerializer,
    &data.accel.as_slice()
);

create_serializer!(
    Data, data,
    AccelLenDataSerializer,
    &data.accel.magnitude()
);

create_serializer!(
    Data, data,
    TimeDataSerializer,
    &data.time
);

create_serializer!(
    Data, data,
    AltitudeDataSerializer,
    &data.pressure_altitude()
);

pub fn id_for_time(data: &Vec<Data>, startid: usize, us: u64) -> Option<usize> {
    if startid >= data.len() {
        return None;
    }

    for i in startid..data.len() {
        let sample = &data[i];
        if sample.time >= us {
            return Some(i);
        }
    }

    return None;
}
