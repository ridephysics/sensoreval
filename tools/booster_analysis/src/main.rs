use itertools::Itertools;
use sensoreval::PlotUtils;
use sensoreval_psim::models::booster::State;
use sensoreval_psim::Model;
use sensoreval_psim::ToImuSample;
use sensoreval_utils::StateUtils;
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
#[serde(deny_unknown_fields)]
struct PlotConfig {
    // input data
    mainconfig: String,
    #[serde(default)]
    startoff: u64,
    #[serde(default)]
    endoff: Option<u64>,
    #[serde(default)]
    accelx_winsz: usize,
    #[serde(default)]
    accely_winsz: usize,
    #[serde(default)]
    theta0d_winsz: usize,
    #[serde(default)]
    theta0dd_winsz: usize,
    #[serde(default)]
    diff_winsz: usize,

    // basic physical parameters
    thetabd: f64,
    rb: f64,
    thetas: f64,
    rs: f64,

    // user-approximated physical parameters
    thetac: Option<f64>,
    friction: Option<f64>,
    rc: Option<f64>,

    // sim
    #[serde(default)]
    stateupdates: Vec<Vec<f64>>,
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

fn calc_theta0_all<F>(accel_needle: f64, calc_accel: F) -> Vec<f64>
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

trait VecInplace<T> {
    fn resize_inplace(self, newlen: usize, value: T) -> Self;
}

impl<T: Clone> VecInplace<T> for Vec<T> {
    fn resize_inplace(mut self, newlen: usize, value: T) -> Self {
        self.resize(newlen, value);
        self
    }
}

trait MedianFilterEx<T> {
    fn consume_ex(&mut self, value: T) -> T;
}

impl<T: Clone + PartialOrd> MedianFilterEx<T> for median::Filter<T> {
    fn consume_ex(&mut self, value: T) -> T {
        // this is the window size and len sounds more natural than is_empty
        #[allow(clippy::len_zero)]
        if self.len() > 0 {
            self.consume(value)
        } else {
            value
        }
    }
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
    let cfg = cfg;
    let samples = cfg.load_data().expect("can't read samples");

    let y_thetab: Vec<_> = samples
        .iter()
        .map(|s| {
            math::normalize_angle(
                (s.time_seconds() - cfg.video.startoff as f64 / 1000.0) * opt.thetabd,
            )
        })
        .collect();
    let y_accelx = samples
        .iter()
        .scan(median::Filter::new(opt.accelx_winsz), |filter, s| {
            Some(filter.consume_ex(s.accel[1]))
        })
        .skip(opt.accelx_winsz / 2)
        .collect::<Vec<_>>()
        .resize_inplace(samples.len(), 0.0);
    let y_accely = samples
        .iter()
        .scan(median::Filter::new(opt.accely_winsz), |filter, s| {
            Some(filter.consume_ex(s.accel[2]))
        })
        .skip(opt.accely_winsz / 2)
        .collect::<Vec<_>>()
        .resize_inplace(samples.len(), 0.0);
    let y_theta0d = samples
        .iter()
        .scan(median::Filter::new(opt.theta0d_winsz), |filter, s| {
            Some(filter.consume_ex(s.gyro[0]))
        })
        .skip(opt.theta0d_winsz / 2)
        .collect::<Vec<_>>()
        .resize_inplace(samples.len(), 0.0);
    let y_theta0dd = samples
        .iter()
        .enumerate()
        .scan(median::Filter::new(opt.theta0dd_winsz), |filter, (i, s)| {
            Some(filter.consume_ex(if i == 0 {
                0.0
            } else {
                (y_theta0d[i - 1] - y_theta0d[i])
                    / (samples[i - 1].time_seconds() - s.time_seconds())
            }))
        })
        .skip(opt.theta0dd_winsz / 2)
        .collect::<Vec<_>>()
        .resize_inplace(samples.len(), 0.0);

    // estimate x1/theta0 for all samples
    let mut y_theta0l = Vec::with_capacity(samples.len());
    let mut y_theta0h = Vec::with_capacity(samples.len());
    let mut y_theta0xl = Vec::with_capacity(samples.len());
    let mut y_theta0xh = Vec::with_capacity(samples.len());
    for (i, _) in samples.iter().enumerate() {
        let thetab = y_thetab[i];
        let theta0d = y_theta0d[i];
        let accelx = y_accelx[i];
        let accely = y_accely[i];
        let theta0dd = y_theta0dd[i];

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
    let y_theta0l = y_theta0l;
    let y_theta0h = y_theta0h;
    let y_theta0xl = y_theta0xl;
    let y_theta0xh = y_theta0xh;

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
            let y: Vec<_> = y_theta0
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
        trace.x(&x).name("filtered");
        trace.line().color(sensoreval_utils::COLOR_E);

        trace.y(&y_theta0d);
        plot.add_trace_to_rowname_ensure(&mut trace, "gyr-e")
            .unwrap();

        trace.y(&y_accelx);
        plot.add_trace_to_rowname_ensure(&mut trace, "acc-n")
            .unwrap();

        trace.y(&y_accely);
        plot.add_trace_to_rowname_ensure(&mut trace, "acc-u")
            .unwrap();
    }

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

    {
        let mut trace = sensoreval_utils::Plot::default_line();
        trace.x(&x).name("estimation");
        trace.line().color(sensoreval_utils::COLOR_E);

        let y: Vec<_> = y_theta0l
            .iter()
            .enumerate()
            .map(|(i, theta0)| theta0 - y_thetab[i])
            .collect();
        trace.y(&y);
        plot.add_trace_to_rowname_ensure(&mut trace, "bo").unwrap();

        let y: Vec<_> = y_theta0d
            .iter()
            .map(|theta0d| theta0d - opt.thetabd)
            .collect();
        trace.y(&y);
        plot.add_trace_to_rowname_ensure(&mut trace, "bod").unwrap();
    }

    // the booster is not moving
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

        // actual: accel data
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

    // simulator
    if opt.rc.is_some() && opt.thetac.is_some() {
        let mut model = sensoreval_psim::models::Booster::new(
            sensoreval_psim::models::BoosterParams {
                objects: vec![sensoreval_psim::models::booster::Object {
                    r: opt.rc.unwrap(),
                    t: opt.thetac.unwrap(),
                    m: 1.0,
                }],
                rb: opt.rb,
                thetas: opt.thetas,
                rs: opt.rs,
                friction: opt.friction,
            },
            0.001,
        );
        let mut timedarray = sensoreval_utils::TimedArray::new(&opt.stateupdates);

        // calculate all states
        let mut states = Vec::with_capacity(samples.len());
        let mut state = ndarray::array![y_thetab[0], opt.thetabd, y_theta0l[0], y_theta0d[0]];
        for (i, s) in samples.iter().enumerate() {
            if i != 0 {
                model.set_dt(s.time_seconds() - samples[i - 1].time_seconds());
                model.step(&mut state);
            }

            states.push(state.clone());

            if let Some(newstate) = timedarray.next(s.time_seconds()) {
                println!("update @ {} with {:?}", s.time_seconds(), newstate);
                for (i, val) in newstate.iter().enumerate() {
                    if !val.is_nan() {
                        state[i] = *val;
                    }
                }
            }
        }

        let y_theta0dd_sim: Vec<_> = samples
            .iter()
            .enumerate()
            .map(|(i, _)| {
                state[State::ThetaB] = y_thetab[i];
                state[State::ThetaBD] = opt.thetabd;
                state[State::Theta0] = y_theta0l[i];
                state[State::Theta0D] = y_theta0d[i];

                if i == 0 {
                    0.0
                } else {
                    model.params().theta0dd(&state)
                }
            })
            .collect();
        let y_diff: Vec<_> = y_theta0dd_sim
            .iter()
            .enumerate()
            .scan(
                median::Filter::new(opt.diff_winsz),
                |filter, (i, theta0dd_sim)| Some(filter.consume_ex(theta0dd_sim - y_theta0dd[i])),
            )
            .skip(opt.diff_winsz / 2)
            .collect::<Vec<_>>()
            .resize_inplace(samples.len(), 0.0);
        let y_r: Vec<_> = y_theta0dd_sim
            .iter()
            .enumerate()
            .map(|(i, theta0dd_sim)| theta0dd_sim - y_theta0dd[i])
            .collect();
        let y_friction: Vec<_> = samples
            .iter()
            .enumerate()
            .map(|(i, _)| y_diff[i] / (y_theta0d[i] - opt.thetabd))
            .collect();

        let mut trace = sensoreval_utils::Plot::default_line();
        trace.x(&x).name("[actual]");
        trace.line().color(sensoreval_utils::COLOR_A);

        trace.y(&y_theta0dd_sim);
        plot.add_trace_to_rowname_ensure(&mut trace, &name_theta0dd)
            .unwrap();

        trace.y(&y_diff);
        plot.add_trace_to_rowname_ensure(&mut trace, "o-diff")
            .unwrap();

        trace.y(&y_r);
        plot.add_trace_to_rowname_ensure(&mut trace, "o-r").unwrap();

        trace.y(&y_friction);
        plot.add_trace_to_rowname_ensure(&mut trace, "o-fv")
            .unwrap();

        let mut trace = sensoreval_utils::Plot::default_line();
        trace.x(&x).name("[sim]");
        trace.line().color(sensoreval_utils::COLOR_A);

        let y: Vec<f64> = states
            .iter()
            .map(|state| math::normalize_angle(state[State::Theta0]))
            .collect();
        trace.y(&y);
        plot.add_trace_to_rowname_ensure(&mut trace, &name_theta0)
            .unwrap();

        let y: Vec<f64> = states.iter().map(|state| state[State::Theta0D]).collect();
        trace.y(&y);
        plot.add_trace_to_rowname_ensure(&mut trace, &name_theta0d)
            .unwrap();

        let mut accel = ndarray::Array1::zeros(3);

        let y: Vec<f64> = states
            .iter()
            .map(|state| {
                model.to_accel(state, &mut accel);
                accel[1]
            })
            .collect();
        trace.y(&y);
        plot.add_trace_to_rowname_ensure(&mut trace, "acc-n")
            .unwrap();

        let y: Vec<f64> = states
            .iter()
            .map(|state| {
                model.to_accel(state, &mut accel);
                accel[2]
            })
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
