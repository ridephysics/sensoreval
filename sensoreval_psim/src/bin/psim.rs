extern crate lapack_src;

use clap::Parser as _;

#[derive(serde::Deserialize)]
struct SimConfig {
    state: Vec<f64>,
    params: sensoreval_psim::models::Params,
}

#[derive(serde::Deserialize)]
struct Config {
    sim: SimConfig,
}

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Config file to use
    config: std::path::PathBuf,

    /// Integration time step in seconds
    #[arg(default_value_t = 0.001)]
    dt: f64,
}

fn main() {
    let cli = Cli::parse();
    let cfgstr = std::fs::read_to_string(&cli.config).unwrap();
    let cfg: Config = toml::from_str(&cfgstr).unwrap();
    let state = ndarray::Array::from(cfg.sim.state);

    sensoreval_psim::run::run_sim(cli.dt, &cfg.sim.params, state);
}
