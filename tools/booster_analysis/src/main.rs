use itertools::Itertools;
use sensoreval::PlotUtils;
use sensoreval_psim::models::booster::State;
use sensoreval_psim::Model;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct PendulumPeriodOpt {
    /// angle at which the pendulum doesn't move (thetad = 0)
    #[structopt(long)]
    theta: f64,
    #[structopt(long)]
    period: f64,
    #[structopt(long)]
    min: usize,
    #[structopt(long)]
    max: usize,
    #[structopt(long)]
    nsteps: usize,
    #[structopt(long, default_value = "0.01")]
    accuracy: f64,
}

fn pendulum_period(opt: &PendulumPeriodOpt) {
    for r in opt.min * opt.nsteps..opt.max * opt.nsteps {
        let r = r as f64 / opt.nsteps as f64;

        let period = math::pendulum_period(r, opt.theta.abs(), math::GRAVITY);
        if (period - opt.period).abs() >= opt.accuracy {
            continue;
        }

        println!("r={} period={}", r, period);
    }
}

#[derive(Debug, StructOpt)]
struct Multi2SingleOpt {
    #[structopt(long, default_value = "0.0,0.0,0.0,0.0")]
    psim_init: math::Array1Opt<f64>,
    #[structopt(long, default_value = "1.0")]
    rb: f64,
    #[structopt(short)]
    objects: Vec<sensoreval_psim::models::booster::Object>,

    #[structopt(long, default_value = "0.01")]
    dt: f64,
}

fn multi2single(opt: &Multi2SingleOpt) {
    let nsamples = (100.0 / opt.dt) as usize;

    let mut model_multi = sensoreval_psim::models::Booster::new(
        sensoreval_psim::models::BoosterParams {
            objects: opt.objects.clone(),
            rb: opt.rb,
            thetas: 0.0,
            rs: 0.5,
            friction: None,
        },
        opt.dt,
    );
    let mut model = sensoreval_psim::models::Booster::new(
        sensoreval_psim::models::BoosterParams {
            objects: vec![sensoreval_psim::models::booster::Object {
                r: model_multi.rc(),
                t: model_multi.thetac(),
                m: 1.0,
            }],
            rb: opt.rb,
            thetas: 0.0,
            rs: 0.5,
            friction: None,
        },
        opt.dt,
    );

    let mut plot = sensoreval_utils::Plot::new("/tmp/sensoreval-plot.html").unwrap();
    let mut ts = Vec::with_capacity(nsamples);
    let mut xs = Vec::with_capacity(nsamples);
    let mut xs_multi = Vec::with_capacity(nsamples);

    let mut x = opt.psim_init.a.clone();
    let mut x_multi = opt.psim_init.a.clone();
    for id in 0..nsamples {
        let t = id as f64 * opt.dt;
        ts.push(t);
        xs.push(x.clone());
        xs_multi.push(x_multi.clone());

        model.step(&mut x);
        model_multi.step(&mut x_multi);
    }

    for arg in &[("single", &xs), ("multi", &xs_multi)] {
        let (name, xs) = *arg;
        let mut trace = sensoreval_utils::Plot::default_line();
        trace.x(&ts).name(name);

        let y: Vec<f64> = xs.iter().map(|x| math::normalize_angle(x[0])).collect();
        trace.y(&y);
        plot.add_trace_to_rowname_ensure(&mut trace, "tb").unwrap();

        let y: Vec<f64> = xs.iter().map(|x| math::normalize_angle(x[2])).collect();
        trace.y(&y);
        plot.add_trace_to_rowname_ensure(&mut trace, "t0").unwrap();

        let y: Vec<f64> = xs.iter().map(|x| x[3]).collect();
        trace.y(&y);
        plot.add_trace_to_rowname_ensure(&mut trace, "t0d").unwrap();

        let y: Vec<f64> = xs
            .iter()
            .enumerate()
            .map(|(i, x)| if i > 0 { x[1] - xs[i - 1][1] } else { 0.0 } / opt.dt)
            .collect();
        trace.y(&y);
        plot.add_trace_to_rowname_ensure(&mut trace, "t0dd")
            .unwrap();
    }

    plot.finish().unwrap();
}

#[derive(Debug, StructOpt)]
struct PlotOpt {
    plotconfig: String,
    #[structopt(long)]
    theta0_lowhigh: bool,
}

#[derive(serde::Deserialize)]
struct PlotConfig {
    #[serde(default)]
    startoff: u64,
    #[serde(default)]
    endoff: Option<u64>,

    thetabd: f64,
    rb: f64,
    friction: Option<f64>,

    thetas: f64,
    rs: f64,

    thetac: Option<f64>,

    mainconfig: String,
}

fn calc_theta0_one<F>(accel_needle: f64, calc_accel: F, other: Option<f64>) -> Option<f64>
where
    F: Fn(f64) -> f64,
{
    let mut theta0 = -std::f64::consts::PI;
    let mut best: Option<(f64, f64)> = None;
    loop {
        let accely = calc_accel(theta0);

        let diff = (accely - accel_needle).abs();
        if other.is_none() || (other.unwrap() - theta0).abs() >= 0.01 {
            match best.as_mut() {
                Some(best) => {
                    if diff < (best.1 - accel_needle).abs() {
                        best.0 = theta0;
                        best.1 = accely;
                    }
                }
                None => {
                    best = Some((theta0, accely));
                }
            }
        }

        theta0 += 0.001;
        if theta0 > std::f64::consts::PI {
            break;
        }
    }

    best.map(|best| best.0)
}

pub fn calc_theta0_all<F>(accel_needle: f64, calc_accel: F) -> Vec<f64>
where
    F: Fn(f64) -> f64,
{
    let mut results = Vec::new();

    let first = calc_theta0_one(accel_needle, &calc_accel, None).unwrap();
    let other = calc_theta0_one(accel_needle, &calc_accel, Some(first));

    results.push(first);
    if let Some(other) = other {
        results.push(other);
    }

    results
}

fn plot(plotopt: &PlotOpt) {
    // load analysis options
    let optdir = std::path::Path::new(&plotopt.plotconfig)
        .parent()
        .expect("can't get parent dir of config");
    let optstr = std::fs::read_to_string(&plotopt.plotconfig).unwrap();
    let mut opt: PlotConfig = toml::from_str(&optstr).unwrap();
    opt.mainconfig = optdir
        .join(std::path::Path::new(&opt.mainconfig))
        .to_str()
        .unwrap()
        .to_string();
    let opt = opt;

    // load main config
    let mut cfg = sensoreval::config::load(&opt.mainconfig).expect("can't load config");
    cfg.video.startoff = opt.startoff;
    cfg.video.endoff = opt.endoff;
    cfg.hud.renderer = sensoreval::config::HudRenderer::Generic;
    let samples = cfg.load_data().expect("can't read samples");

    let mut iir_theta0dd = math::IIR::new(16.0);
    let mut iir_theta0 = math::IIR::new(16.0);
    let mut y_thetab = Vec::with_capacity(samples.len());
    let mut y_accelx = Vec::with_capacity(samples.len());
    let mut y_accely = Vec::with_capacity(samples.len());
    let mut y_theta0l = Vec::with_capacity(samples.len());
    let mut y_theta0h = Vec::with_capacity(samples.len());
    let mut y_theta0d = Vec::with_capacity(samples.len());
    let mut y_theta0dd = Vec::with_capacity(samples.len());
    let mut y_theta0xl = Vec::with_capacity(samples.len());
    let mut y_theta0xh = Vec::with_capacity(samples.len());

    // estimate x1/theta0 for all samples
    for (i, s) in samples.iter().enumerate() {
        let theta0d = s.gyro[0];
        y_theta0d.push(theta0d);

        let accelx = s.accel[1];
        y_accelx.push(accelx);

        let accely = s.accel[2];
        y_accely.push(accely);

        let thetab = math::normalize_angle(
            (s.time_seconds() - cfg.video.startoff as f64 / 1000.0) * opt.thetabd,
        );
        y_thetab.push(thetab);

        let theta0dd = if i == 0 {
            0.0
        } else {
            (y_theta0d[i - 1] - theta0d)
                / (samples[i - 1].time_seconds() - samples[i].time_seconds())
        };
        let theta0dd = iir_theta0dd.next(theta0dd);
        y_theta0dd.push(theta0dd);

        let all_x = calc_theta0_all(accelx, |theta0| {
            -opt.rb * opt.thetabd.powi(2) * (thetab - theta0).sin()
                - opt.rs * theta0d.powi(2) * opt.thetas.sin()
                + opt.rs * theta0dd * opt.thetas.cos()
                + math::GRAVITY * theta0.sin()
        });
        let all_y = calc_theta0_all(accely, |theta0| {
            opt.rb * opt.thetabd.powi(2) * (thetab - theta0).cos()
                + opt.rs * theta0d.powi(2) * opt.thetas.cos()
                + opt.rs * theta0dd * opt.thetas.sin()
                + math::GRAVITY * theta0.cos()
        });

        if !plotopt.theta0_lowhigh {
            let mut all: Vec<f64> = Vec::new();
            all.extend(&all_x);
            all.extend(&all_y);

            let (_, best_a, best_b) = (0..all.len())
                .permutations(2)
                .unique()
                .map(|perm| {
                    let ida = perm[0];
                    let idb = perm[1];
                    ((all[ida] - all[idb]).abs(), ida, idb)
                })
                .min_by(|x, y| x.0.partial_cmp(&y.0).unwrap())
                .unwrap();
            let best = (all[best_a] + all[best_b]) / 2.0;
            let best = iir_theta0.next(best);
            y_theta0l.push(best);
        } else {
            y_theta0xl.push(
                *all_x
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            );
            y_theta0xh.push(
                *all_x
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            );

            y_theta0l.push(
                *all_y
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            );
            y_theta0h.push(
                *all_y
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            );
        }
    }

    let mut plot = sensoreval_utils::Plot::new("/tmp/sensoreval-plot.html").unwrap();
    let x: Vec<f64> = samples.iter().map(|sample| sample.time_seconds()).collect();
    plot.add_measurements(&samples, &x).unwrap();

    let name_thetab = format!("x{}", State::ThetaB.id());
    let name_thetabd = format!("x{}", State::ThetaBD.id());
    let name_theta0 = format!("x{}", State::Theta0.id());
    let name_theta0d = format!("x{}", State::Theta0D.id());
    let name_theta0dd = format!("x{}dd", State::Theta0.id());

    // measurement: theta0dd
    {
        let mut trace = sensoreval_utils::Plot::default_line();
        trace.x(&x).name("measurement");
        trace.line().color(sensoreval_utils::COLOR_M);

        trace.y(&y_theta0dd);
        plot.add_trace_to_rowname_ensure(&mut trace, &name_theta0dd)
            .unwrap();
    }

    let colors = [sensoreval_utils::COLOR_E, "#00E600"];
    let names = ["estimation0", "estimation1"];

    if !y_theta0xl.is_empty() || !y_theta0xh.is_empty() {
        for (i, y_theta0x) in [&y_theta0xl, &y_theta0xh].iter().enumerate() {
            let mut trace = sensoreval_utils::Plot::default_line();
            trace.x(&x).name(names[i]);
            trace.line().color(colors[i]);

            trace.y(&y_theta0x);
            plot.add_trace_to_rowname_ensure(&mut trace, "t0x").unwrap();
        }
    }

    let mut y_theta0_lh = vec![&y_theta0l];

    if !y_theta0h.is_empty() {
        y_theta0_lh.push(&y_theta0h);
    }

    for (i, y_theta0) in y_theta0_lh.iter().enumerate() {
        println!();
        println!("[y_theta0_{}]", i);

        let mut trace = sensoreval_utils::Plot::default_line();
        trace.x(&x).name(names[i]);
        trace.line().color(colors[i]);

        trace.y(&y_theta0);
        plot.add_trace_to_rowname_ensure(&mut trace, &name_theta0)
            .unwrap();

        let avg_theta0 = y_theta0.iter().sum::<f64>() / y_theta0.len() as f64;
        println!("avg_theta0: {}", avg_theta0);

        if let Some(thetac) = opt.thetac {
            // rc
            let mut trace = trace.clone();
            let mut n: usize = 0;
            let mut sum = 0.0;
            let y: Vec<f64> = y_theta0
                .iter()
                .enumerate()
                .map(|(i, theta0)| {
                    let angle1 = math::normalize_angle(theta0 + thetac);
                    let angle2 = math::normalize_angle(y_thetab[i] - theta0 - thetac);
                    let mut divisor = y_theta0dd[i];
                    if let Some(friction) = opt.friction {
                        divisor += friction * (y_theta0d[i] - opt.thetabd);
                    }

                    let r = (opt.rb * opt.thetabd.powi(2) * angle2.sin()
                        - math::GRAVITY * angle1.sin())
                        / divisor;
                    if y_theta0dd[i] > 1.0 && r.is_sign_positive() && r.is_normal() {
                        n += 1;
                        sum += r;

                        r
                    } else {
                        0.0
                    }
                })
                .collect();
            trace.y(&y);
            plot.add_trace_to_rowname_ensure(&mut trace, "rc").unwrap();

            println!("rc_avg={}", sum / n as f64);
        }
    }
    println!();

    {
        let mut trace = sensoreval_utils::Plot::default_line();
        trace.x(&x).name("[actual]");
        trace.line().color(sensoreval_utils::COLOR_A);

        // actual: x0 / thetaB
        trace.y(&y_thetab);
        plot.add_trace_to_rowname_ensure(&mut trace, &name_thetab)
            .unwrap();

        // actual: x1 / thetaBD
        let y_thetabd: Vec<f64> = (0..samples.len()).map(|_| opt.thetabd).collect();
        trace.y(&y_thetabd);
        plot.add_trace_to_rowname_ensure(&mut trace, &name_thetabd)
            .unwrap();

        // actual: x3 / theta0D
        trace.y(&y_theta0d);
        plot.add_trace_to_rowname_ensure(&mut trace, &name_theta0d)
            .unwrap();
    }

    // actual: accel data, if the booster is not moving
    if !opt.thetabd.is_normal() {
        let avg_theta0x = samples
            .iter()
            .map(|s| (s.accel[1] / math::GRAVITY).asin())
            .sum::<f64>()
            / samples.len() as f64;
        let avg_theta0y = samples
            .iter()
            .map(|s| (s.accel[2] / math::GRAVITY).acos())
            .sum::<f64>()
            / samples.len() as f64;
        let avg_theta0 = (avg_theta0x + avg_theta0y) / 2.0;
        println!(
            "avg_theta0x={} avg_theta0y={} avg_theta0={}",
            avg_theta0x, avg_theta0y, avg_theta0
        );

        let mut trace = sensoreval_utils::Plot::default_line();
        trace.x(&x).name("[actual]");
        trace.line().color(sensoreval_utils::COLOR_A);

        let y: Vec<f64> = (0..samples.len())
            .map(|_| math::GRAVITY * avg_theta0.sin())
            .collect();
        trace.y(&y);
        plot.add_trace_to_rowname_ensure(&mut trace, "acc-n")
            .unwrap();

        let y: Vec<f64> = (0..samples.len())
            .map(|_| math::GRAVITY * avg_theta0.cos())
            .collect();
        trace.y(&y);
        plot.add_trace_to_rowname_ensure(&mut trace, "acc-u")
            .unwrap();
    }

    plot.finish().unwrap();
}

#[derive(Debug, StructOpt)]
enum Opt {
    #[structopt(name = "pendulum_period")]
    PendulumPeriod(PendulumPeriodOpt),
    #[structopt(name = "multi2single")]
    Multi2Single(Multi2SingleOpt),
    Plot(PlotOpt),
}

fn main() {
    let opt = Opt::from_args();

    match &opt {
        Opt::PendulumPeriod(o) => pendulum_period(o),
        Opt::Multi2Single(o) => multi2single(o),
        Opt::Plot(o) => plot(o),
    }
}
