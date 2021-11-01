use anyhow::anyhow;
use anyhow::Context as _;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    coeff: f64,
}

pub fn apply(config: &Config, dataset: &mut crate::Dataset) -> anyhow::Result<()> {
    let mut pressure_all = dataset
        .get_kind_mut("pressure")
        .ok_or_else(|| anyhow!("can't find pressure"))?
        .view_mut()
        .into_dimensionality::<ndarray::Ix1>()
        .context("unsupported pressure dimension")?;

    let mut pressure_prev = None;
    for pressure_dst in &mut pressure_all {
        let mut pressure = *pressure_dst;

        // apply pressure coefficient
        if config.coeff > 0. {
            if let Some(pressure_prev) = pressure_prev {
                pressure = (pressure_prev * (config.coeff - 1.0) + pressure)
                    / config.coeff;
            }
        }

        pressure_prev = Some(pressure);
        *pressure_dst = pressure;
    }

    Ok(())
}
