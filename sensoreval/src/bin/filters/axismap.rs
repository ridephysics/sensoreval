use anyhow::anyhow;
use anyhow::Context as _;

/// map sensor axes. index: destination, value: source + 1, can be negative
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AxisMap(Vec<isize>);

impl Default for AxisMap {
    fn default() -> Self {
        Self(vec![1, 2, 3])
    }
}

impl AxisMap {
    /// copy one axis
    #[inline(always)]
    pub fn copy_single<A, T>(&self, dst: &mut A, src: &[T], dstidx: usize)
    where
        A: std::ops::IndexMut<usize, Output = T>,
        T: Copy + std::ops::Neg<Output = T>,
    {
        let mut srcidx = self.0[dstidx].abs() as usize;
        assert!(srcidx != 0);
        srcidx -= 1;

        let mut tmp = src[srcidx];
        if self.0[dstidx] < 0 {
            tmp = -tmp;
        }
        dst[dstidx] = tmp;
    }

    /// copy all axes
    #[inline(always)]
    pub fn copy<A, T>(&self, dst: &mut A, src: &[T])
    where
        A: std::ops::IndexMut<usize, Output = T>,
        T: Copy + std::ops::Neg<Output = T>,
    {
        for i in 0..self.0.len() {
            self.copy_single(dst, src, i);
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    map: AxisMap,
}

pub fn apply(config: &Config, dataset: &mut crate::Dataset) -> anyhow::Result<()> {
    let mut accel_all = dataset
        .get_kind_mut("accel")
        .ok_or_else(|| anyhow!("can't find accel"))?
        .view_mut()
        .into_dimensionality::<ndarray::Ix2>()
        .context("unsupported accel dimension")?;
    for mut accel in accel_all.lanes_mut(ndarray::Axis(1)) {
        let copy = &accel.as_slice().unwrap().to_vec();
        config.map.copy(&mut accel, copy);
    }

    let mut gyro_all = dataset
        .get_kind_mut("gyro")
        .ok_or_else(|| anyhow!("can't find gyro"))?
        .view_mut()
        .into_dimensionality::<ndarray::Ix2>()
        .context("unsupported gyro dimension")?;
    for mut gyro in gyro_all.lanes_mut(ndarray::Axis(1)) {
        let copy = &gyro.as_slice().unwrap().to_vec();
        config.map.copy(&mut gyro, copy);
    }

    let mut mag_all = dataset
        .get_kind_mut("mag")
        .ok_or_else(|| anyhow!("can't find mag"))?
        .view_mut()
        .into_dimensionality::<ndarray::Ix2>()
        .context("unsupported mag dimension")?;
    for mut mag in mag_all.lanes_mut(ndarray::Axis(1)) {
        let copy = &mag.as_slice().unwrap().to_vec();
        config.map.copy(&mut mag, copy);
    }

    Ok(())
}
