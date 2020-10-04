use clap::value_t;
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

fn vararg<'a, 'b>(name: &'a str, dval: &'a str) -> clap::Arg<'a, 'b> {
    clap::Arg::with_name(name)
        .help(name)
        .long(name)
        .default_value(dval)
        .takes_value(true)
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
        .subcommand(
            clap::SubCommand::with_name("double_pendulum")
                .arg(vararg("m1", "1.0"))
                .arg(vararg("m2", "1.0"))
                .arg(vararg("l1", "1.0"))
                .arg(vararg("l2", "1.0"))
                .arg(vararg("t1", "3.0"))
                .arg(vararg("t1d", "0.0"))
                .arg(vararg("t2", "2.0"))
                .arg(vararg("t2d", "0.0")),
        )
        .subcommand(
            clap::SubCommand::with_name("pendulum")
                .arg(vararg("l", "1.0"))
                .arg(vararg("t", "3.0"))
                .arg(vararg("td", "0.0")),
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
            let m1: f64 = value_t!(sub_m, "m1", f64).unwrap();
            let m2: f64 = value_t!(sub_m, "m2", f64).unwrap();
            let l1: f64 = value_t!(sub_m, "l1", f64).unwrap();
            let l2: f64 = value_t!(sub_m, "l2", f64).unwrap();
            let t1: f64 = value_t!(sub_m, "t1", f64).unwrap();
            let t1d: f64 = value_t!(sub_m, "t1d", f64).unwrap();
            let t2: f64 = value_t!(sub_m, "t2", f64).unwrap();
            let t2d: f64 = value_t!(sub_m, "t2d", f64).unwrap();

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
            let l: f64 = value_t!(sub_m, "l", f64).unwrap();
            let t: f64 = value_t!(sub_m, "t", f64).unwrap();
            let td: f64 = value_t!(sub_m, "td", f64).unwrap();
            let params = sensoreval_psim::models::PendulumParams {
                radius: l,
                sensor_pos: 0.0,
                motor: None,
            };

            gui.set_callback(Some(GuiCallback::new(
                sensoreval_psim::models::Pendulum::new(params, dt),
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
