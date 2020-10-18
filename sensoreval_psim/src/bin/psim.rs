extern crate lapack_src;

#[derive(serde::Deserialize)]
struct SimConfig {
    state: Vec<f64>,
    params: sensoreval_psim::models::Params,
}

#[derive(serde::Deserialize)]
struct Config {
    sim: SimConfig,
}

fn main() {
    let app_m = clap::App::new("psim")
        .setting(clap::AppSettings::AllowLeadingHyphen)
        .arg(
            clap::Arg::with_name("dt")
                .default_value("0.001")
                .long("dt")
                .takes_value(true)
                .help("integration time step in seconds"),
        )
        .arg(
            clap::Arg::with_name("CONFIG")
                .help("config file to use")
                .required(true)
                .index(1),
        )
        .get_matches();
    let dt: f64 = app_m
        .value_of("dt")
        .unwrap()
        .parse()
        .expect("can't parse dt");
    let cfgname = app_m.value_of("CONFIG").unwrap();

    let cfgstr = std::fs::read_to_string(cfgname).unwrap();
    let cfg: Config = toml::from_str(&cfgstr).unwrap();
    let state = ndarray::Array::from(cfg.sim.state);

    sensoreval_psim::run::run_sim(dt, &cfg.sim.params, state);
}
