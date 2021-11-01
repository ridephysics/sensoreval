use anyhow::anyhow;
use anyhow::Context as _;
use bincode::Options as _;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct MpuCfgDump {
    mputype: u8,
    magtype: u8,
    gyro_fsr: u8,
    accel_fsr: u8,
    mag_sens_adj: [i16; 3],
}

const MPU_TYPE_MPU6050: u8 = 0;
const MPU_TYPE_MPU6500: u8 = 1;

const MAG_TYPE_AK8975: u8 = 1;
const MAG_TYPE_AK8963: u8 = 2;

impl MpuCfgDump {
    /// Get gyro sensitivity scale factor.
    ///
    /// Used for conversion from hardware units to dps.
    pub fn gyro_sens(&self) -> anyhow::Result<f64> {
        const GYRO_FSR_250DPS: u8 = 0;
        const GYRO_FSR_500DPS: u8 = 1;
        const GYRO_FSR_1000DPS: u8 = 2;
        const GYRO_FSR_2000DPS: u8 = 3;

        Ok(match self.gyro_fsr {
            GYRO_FSR_250DPS => 131.0,
            GYRO_FSR_500DPS => 65.5,
            GYRO_FSR_1000DPS => 32.8,
            GYRO_FSR_2000DPS => 16.4,
            _ => return Err(anyhow!("unsupported gyro FSR: {}", self.gyro_fsr)),
        })
    }

    /// Get accel sensitivity scale factor.
    ///
    /// Used for conversion from hardware units to g's.
    pub fn accel_sens(&self) -> anyhow::Result<u16> {
        const ACCEL_FSR_2G: u8 = 0;
        const ACCEL_FSR_4G: u8 = 1;
        const ACCEL_FSR_8G: u8 = 2;
        const ACCEL_FSR_16G: u8 = 3;

        Ok(match self.accel_fsr {
            ACCEL_FSR_2G => 16384,
            ACCEL_FSR_4G => 8192,
            ACCEL_FSR_8G => 4096,
            ACCEL_FSR_16G => 2048,
            _ => return Err(anyhow!("unsupported accel FSR: {}", self.accel_fsr)),
        })
    }

    pub fn temp_sens(&self) -> anyhow::Result<u16> {
        Ok(match self.mputype {
            MPU_TYPE_MPU6050 => 340,
            MPU_TYPE_MPU6500 => 321,
            _ => return Err(anyhow!("unsupported mputype: {}", self.mputype)),
        })
    }

    pub fn temp_offset(&self) -> anyhow::Result<i16> {
        Ok(match self.mputype {
            MPU_TYPE_MPU6050 => -521,
            MPU_TYPE_MPU6500 => 0,
            _ => return Err(anyhow!("unsupported mputype: {}", self.mputype)),
        })
    }

    /// Get the magnetometer full-scale range.
    ///
    /// Used for conversion from hardware units to uT's.
    pub fn mag_fsr(&self) -> anyhow::Result<u16> {
        Ok(match self.magtype {
            MAG_TYPE_AK8975 => 9830,
            MAG_TYPE_AK8963 => 4915,
            _ => return Err(anyhow!("unsupported magtype: {}", self.magtype)),
        })
    }

    pub fn gyro_to_dps(&self, raw: &[i16; 3]) -> anyhow::Result<[f64; 3]> {
        let sens = self.gyro_sens()? as f64;
        Ok([
            (raw[0] as f64) / sens,
            (raw[1] as f64) / sens,
            (raw[2] as f64) / sens,
        ])
    }

    pub fn accel_to_g(&self, raw: &[i16; 3]) -> anyhow::Result<[f64; 3]> {
        let sens = self.accel_sens()? as f64;
        Ok([
            (raw[0] as f64) / sens,
            (raw[1] as f64) / sens,
            (raw[2] as f64) / sens,
        ])
    }

    pub fn temp_to_degc(&self, raw: i16) -> anyhow::Result<f64> {
        let sens = self.temp_sens()? as f64;
        let offset = self.temp_offset()? as f64;
        Ok((((raw as f64) - offset) / sens) + 21.0)
    }

    pub fn mag_to_ut(&self, regs: &[u8; 8]) -> anyhow::Result<[f64; 3]> {
        const AKM_DATA_READY: u8 = 0x01;
        const AKM_DATA_OVERRUN: u8 = 0x02;
        const AKM_OVERFLOW: u8 = 0x80;
        const AKM_DATA_ERROR: u8 = 0x40;

        let fsr = self.mag_fsr()? as f64;
        let status1 = regs[0];
        let status2 = regs[7];

        match self.magtype {
            MAG_TYPE_AK8975 => {
                /* AK8975 doesn't have the overrun error bit. */
                if (status1 & AKM_DATA_READY) == 0 {
                    return Err(anyhow!("data is not ready"));
                }
                if (status2 & AKM_OVERFLOW) != 0 {
                    return Err(anyhow!("data overflow"));
                }
                if (status2 & AKM_DATA_ERROR) != 0 {
                    return Err(anyhow!("data error"));
                }
            }
            MAG_TYPE_AK8963 => {
                /* AK8963 doesn't have the data read error bit. */
                if (status1 & AKM_DATA_READY) == 0 {
                    return Err(anyhow!("data is not ready"));
                }
                if (status1 & AKM_DATA_OVERRUN) != 0 {
                    return Err(anyhow!("data overrun"));
                }
                if (status2 & AKM_OVERFLOW) != 0 {
                    return Err(anyhow!("data overflow"));
                }
            }
            _ => return Err(anyhow!("unsupported magtype: {}", self.magtype)),
        }

        let mut raw = [
            i16::from_le_bytes([regs[1], regs[2]]),
            i16::from_le_bytes([regs[3], regs[4]]),
            i16::from_le_bytes([regs[5], regs[6]]),
        ];

        // apply factory calibration
        for (raw, mag_sens_adj) in raw.iter_mut().zip(self.mag_sens_adj.iter()) {
            *raw = (((*raw as i32) * (*mag_sens_adj as i32)) >> 8).try_into()?;
        }

        let mut res = [
            (raw[0] as f64) / (32768.0 / fsr),
            (raw[1] as f64) / (32768.0 / fsr),
            (raw[2] as f64) / (32768.0 / fsr),
        ];

        // convert to the same coordinate system as accel and gyro
        res.swap(0, 1);
        res[2] = -res[2];

        Ok(res)
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct Bmp2CalibParamLegacy {
    dig_t1: u16,
    dig_t2: i16,
    dig_t3: i16,
    dig_p1: u16,
    dig_p2: i16,
    dig_p3: i16,
    dig_p4: i16,
    dig_p5: i16,
    dig_p6: i16,
    dig_p7: i16,
    dig_p8: i16,
    dig_p9: i16,
    t_fine: i32,
}

impl Bmp2CalibParamLegacy {
    fn to_native(&self) -> bmp2_sys::bmp2_calib_param {
        bmp2_sys::bmp2_calib_param {
            dig_t1: self.dig_t1,
            dig_t2: self.dig_t2,
            dig_t3: self.dig_t3,
            dig_p1: self.dig_p1,
            dig_p2: self.dig_p2,
            dig_p3: self.dig_p3,
            dig_p4: self.dig_p4,
            dig_p5: self.dig_p5,
            dig_p6: self.dig_p6,
            dig_p7: self.dig_p7,
            dig_p8: self.dig_p8,
            dig_p9: self.dig_p9,
            dig_p10: 0,
            t_fine: self.t_fine,
        }
    }
}

fn bmp2_parse_sensor_data(raw: &[u8; 6]) -> anyhow::Result<bmp2_sys::bmp2_uncomp_data> {
    let mut bmp2_uncomp_data = std::mem::MaybeUninit::<bmp2_sys::bmp2_uncomp_data>::uninit();
    let ret =
        unsafe { bmp2_sys::bmp2_parse_sensor_data(raw.as_ptr(), bmp2_uncomp_data.as_mut_ptr()) };
    if ret == 0 {
        Ok(unsafe { bmp2_uncomp_data.assume_init() })
    } else {
        Err(anyhow!("can't parse sensor data: {}", ret))
    }
}

fn bmp2_compensate_data(
    uncomp_data: &bmp2_sys::bmp2_uncomp_data,
    calib_param: &mut bmp2_sys::bmp2_calib_param,
) -> anyhow::Result<bmp2_sys::bmp2_data> {
    let mut comp_data = std::mem::MaybeUninit::<bmp2_sys::bmp2_data>::uninit();
    let ret =
        unsafe { bmp2_sys::bmp2_compensate_data(uncomp_data, comp_data.as_mut_ptr(), calib_param) };
    if ret == 0 {
        Ok(unsafe { comp_data.assume_init() })
    } else {
        Err(anyhow!("can't compensate sensor data: {}", ret))
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct SentralPtHeader {
    mpu_cfg_dump: MpuCfgDump,
    bmp2_calib_param: Bmp2CalibParamLegacy,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct SentralPtSample {
    accel_0: [u8; 2],
    accel_1: [u8; 2],
    accel_2: [u8; 2],

    temp: [u8; 2],

    gyro_0: [u8; 2],
    gyro_1: [u8; 2],
    gyro_2: [u8; 2],

    mag: [u8; 8],

    time_mpu: u64,

    bmp2_raw: [u8; 6],
    time_bmp: u64,
}

impl SentralPtSample {
    pub fn accel(&self) -> [i16; 3] {
        [
            i16::from_be_bytes(self.accel_0),
            i16::from_be_bytes(self.accel_1),
            i16::from_be_bytes(self.accel_2),
        ]
    }

    pub fn gyro(&self) -> [i16; 3] {
        [
            i16::from_be_bytes(self.gyro_0),
            i16::from_be_bytes(self.gyro_1),
            i16::from_be_bytes(self.gyro_2),
        ]
    }

    pub fn mag(&self) -> &[u8; 8] {
        &self.mag
    }

    pub fn temp(&self) -> i16 {
        i16::from_be_bytes(self.temp)
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    path: std::path::PathBuf,
}

pub fn apply(config: &Config, dataset: &mut crate::Dataset) -> anyhow::Result<()> {
    let bincode_options = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_little_endian();
    let header_size = bincode_options
        .serialized_size(&SentralPtHeader::default())
        .unwrap();
    let sample_size = bincode_options
        .serialized_size(&SentralPtSample::default())
        .unwrap();

    let file = std::fs::File::open(&config.path).context("can't open IMU data file")?;
    let metadata = file
        .metadata()
        .context("can't get metadata of IMU data file")?;
    let mut reader = std::io::BufReader::new(file);

    if metadata.len() < header_size {
        return Err(anyhow!("file is too small to even have a header"));
    }

    let all_samples_size = metadata.len() - header_size;
    if all_samples_size % sample_size != 0 {
        log::warn!("IMU data file has an incomplete sample at the end");
    }
    let num_samples: usize = (all_samples_size / sample_size)
        .try_into()
        .context("number of samples doesn't fit into usize")?;

    let header: SentralPtHeader = bincode_options
        .deserialize_from(&mut reader)
        .context("can't deserialize sentral_pt IMU data header")?;
    let mut bmp2_calib_param = header.bmp2_calib_param.to_native();

    let mut times_all = ndarray::Array::zeros(num_samples);
    let mut accel_all = ndarray::Array::zeros((num_samples, 3));
    let mut gyro_all = ndarray::Array::zeros((num_samples, 3));
    let mut mag_all = ndarray::Array::zeros((num_samples, 3));
    let mut pressure_all = ndarray::Array::zeros(num_samples);
    let mut temp_baro_all = ndarray::Array::zeros(num_samples);
    let mut temp_mpu_all = ndarray::Array::zeros(num_samples);

    for i in 0..num_samples {
        let sample: SentralPtSample = bincode_options
            .deserialize_from(&mut reader)
            .context("can't deserialize sentral_pt IMU data sample")?;
        let mut accel = header.mpu_cfg_dump.accel_to_g(&sample.accel())?;
        for val in &mut accel {
            // g -> m/s^2
            *val *= math::GRAVITY;
        }

        let mut gyro = header.mpu_cfg_dump.gyro_to_dps(&sample.gyro())?;
        for val in &mut gyro {
            // dps -> rad/s
            *val = val.to_radians();
        }

        let mag = header.mpu_cfg_dump.mag_to_ut(sample.mag())?;
        let mpu_temp = header.mpu_cfg_dump.temp_to_degc(sample.temp())?;
        let bmp2_uncomp_data = bmp2_parse_sensor_data(&sample.bmp2_raw)?;
        let bmp2_comp_data = bmp2_compensate_data(&bmp2_uncomp_data, &mut bmp2_calib_param)?;

        times_all[i] = sample.time_mpu;
        accel_all[[i, 0]] = accel[0];
        accel_all[[i, 1]] = accel[1];
        accel_all[[i, 2]] = accel[2];
        gyro_all[[i, 0]] = gyro[0];
        gyro_all[[i, 1]] = gyro[1];
        gyro_all[[i, 2]] = gyro[2];
        mag_all[[i, 0]] = mag[0];
        mag_all[[i, 1]] = mag[1];
        mag_all[[i, 2]] = mag[2];

        // Pa -> hPa
        pressure_all[i] = bmp2_comp_data.pressure / 100.0;

        temp_baro_all[i] = bmp2_comp_data.temperature;
        temp_mpu_all[i] = mpu_temp;
    }

    *dataset = crate::Dataset::new(times_all);
    dataset.add_kind("accel", accel_all.into_dyn());
    dataset.add_kind("gyro", gyro_all.into_dyn());
    dataset.add_kind("mag", mag_all.into_dyn());
    dataset.add_kind("pressure", pressure_all.into_dyn());
    dataset.add_kind("temp_baro", temp_baro_all.into_dyn());
    dataset.add_kind("temp_mpu", temp_mpu_all.into_dyn());

    Ok(())
}
