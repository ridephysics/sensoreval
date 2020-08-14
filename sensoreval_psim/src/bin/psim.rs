use sensoreval_graphics::utils::CairoEx;
use sensoreval_graphics::utils::ToUtilFont;
use sensoreval_psim::Model;

extern crate lapack_src;

type State = std::sync::Arc<std::sync::RwLock<ndarray::Array1<f64>>>;
struct GuiCallback<F> {
    state: State,
    render_state: F,
}

impl<F: Fn(&mut cairo::Context, &State)> GuiCallback<F> {
    pub fn new<M: 'static + Model + Send + Sync>(
        mut model: M,
        initial: ndarray::Array1<f64>,
        render_state: F,
    ) -> Self {
        let o = Self {
            state: std::sync::Arc::new(std::sync::RwLock::new(initial)),
            render_state,
        };

        let state_lock = o.state.clone();
        std::thread::spawn(move || {
            let tstart = std::time::Instant::now();
            let mut idx = 0u64;
            let mut x = state_lock.read().unwrap().clone();
            let dt = model.dt();

            fn secs_since_last_calc(tstart: &std::time::Instant, idx: &u64, dt: &f64) -> f64 {
                tstart.elapsed().as_secs_f64() - *idx as f64 * *dt
            }

            loop {
                let niter = (secs_since_last_calc(&tstart, &idx, &dt) / dt) as u64;

                for _ in 0..niter as usize {
                    model.step(&mut x);
                }

                idx += niter;
                state_lock.write().unwrap().assign(&x);

                let waittime = secs_since_last_calc(&tstart, &idx, &dt);
                if waittime > 0.0 {
                    std::thread::sleep(std::time::Duration::from_secs_f64(waittime));
                }
            }
        });

        o
    }
}

impl<F: Fn(&mut cairo::Context, &State)> sensoreval_gui::Callback for GuiCallback<F> {
    fn render(&mut self, _ctx: &mut sensoreval_gui::RuntimeContext, cr: &mut cairo::Context) {
        // clear
        cr.set_source_rgba_u32(0x00000000);
        cr.clear();

        cr.set_source_rgba_u32(0xffffffff);
        (self.render_state)(cr, &self.state);
    }
}

fn main() {
    let app_m = clap::App::new("psim")
        .arg(
            clap::Arg::with_name("dt")
                .default_value("0.001")
                .long("dt")
                .takes_value(true)
                .help("integration time step in seconds"),
        )
        .subcommand(
            clap::SubCommand::with_name("double_pendulum")
                .arg(
                    clap::Arg::with_name("m1")
                        .help("m1")
                        .long("m1")
                        .default_value("1.0"),
                )
                .arg(
                    clap::Arg::with_name("m2")
                        .help("m2")
                        .long("m2")
                        .default_value("1.0"),
                )
                .arg(
                    clap::Arg::with_name("l1")
                        .help("l1")
                        .long("l1")
                        .default_value("1.0"),
                )
                .arg(
                    clap::Arg::with_name("l2")
                        .help("l2")
                        .long("l2")
                        .default_value("1.0"),
                )
                .arg(
                    clap::Arg::with_name("t1")
                        .help("theta1")
                        .long("t1")
                        .default_value("3.0"),
                )
                .arg(
                    clap::Arg::with_name("t1d")
                        .help("theta1-dot")
                        .long("t1d")
                        .default_value("0.0"),
                )
                .arg(
                    clap::Arg::with_name("t2")
                        .help("theta2")
                        .long("t2")
                        .default_value("2.0"),
                )
                .arg(
                    clap::Arg::with_name("t2d")
                        .help("theta2-dot")
                        .long("t2d")
                        .default_value("0.0"),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("pendulum")
                .arg(
                    clap::Arg::with_name("l")
                        .help("length")
                        .long("l")
                        .default_value("1.0"),
                )
                .arg(
                    clap::Arg::with_name("t")
                        .help("theta")
                        .long("t")
                        .default_value("3.0"),
                )
                .arg(
                    clap::Arg::with_name("td")
                        .help("theta-dot")
                        .long("td")
                        .default_value("0.0"),
                ),
        )
        .get_matches();
    let dt: f64 = app_m
        .value_of("dt")
        .unwrap()
        .parse()
        .expect("can't parse dt");
    let mut gui = sensoreval_gui::Context::default();

    let mut font = pango::FontDescription::new();
    font.set_family("Archivo Black");
    font.set_absolute_size(30.0 * f64::from(pango::SCALE));
    let font = font.to_utilfont();

    match app_m.subcommand() {
        ("double_pendulum", Some(sub_m)) => {
            let m1: f64 = sub_m
                .value_of("m1")
                .unwrap()
                .parse()
                .expect("can't parse m1");
            let m2: f64 = sub_m
                .value_of("m2")
                .unwrap()
                .parse()
                .expect("can't parse m2");
            let l1: f64 = sub_m
                .value_of("l1")
                .unwrap()
                .parse()
                .expect("can't parse l1");
            let l2: f64 = sub_m
                .value_of("l2")
                .unwrap()
                .parse()
                .expect("can't parse l2");
            let t1: f64 = sub_m
                .value_of("t1")
                .unwrap()
                .parse()
                .expect("can't parse t1");
            let t1d: f64 = sub_m
                .value_of("t1d")
                .unwrap()
                .parse()
                .expect("can't parse t1d");
            let t2: f64 = sub_m
                .value_of("t2")
                .unwrap()
                .parse()
                .expect("can't parse t2");
            let t2d: f64 = sub_m
                .value_of("t2d")
                .unwrap()
                .parse()
                .expect("can't parse t2d");

            gui.set_callback(Some(GuiCallback::new(
                sensoreval_psim::models::DoublePendulum::new(m1, m2, l1, l2, dt),
                ndarray::array![t1d, t2d, t1, t2],
                move |cr, state| {
                    let state = state.read().unwrap();
                    let m1a = state[2];
                    let m2a = state[3];
                    drop(state);
                    sensoreval_graphics::double_pendulum_2d::draw(cr, m1a, m2a, l1, l2);
                },
            )));
        }
        ("pendulum", Some(sub_m)) => {
            let l: f64 = sub_m.value_of("l").unwrap().parse().expect("can't parse l");
            let t: f64 = sub_m.value_of("t").unwrap().parse().expect("can't parse t");
            let td: f64 = sub_m
                .value_of("td")
                .unwrap()
                .parse()
                .expect("can't parse td");

            gui.set_callback(Some(GuiCallback::new(
                sensoreval_psim::models::Pendulum::new(l, dt),
                ndarray::array![t, td],
                move |cr, state| {
                    let state = state.read().unwrap().clone();
                    sensoreval_graphics::pendulum_2d::draw(cr, state[0]);

                    font.draw(cr, &format!("t: {}\ntd: {}", state[0], state[1]));
                },
            )));
        }
        _ => panic!("invalid subcommand"),
    }

    gui.set_timer_ms(15);
    gui.start().unwrap();
}
