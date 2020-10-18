use sensoreval_graphics::utils::CairoEx;
use sensoreval_graphics::utils::ToUtilFont;
use sensoreval_psim::DrawState;
use sensoreval_psim::Model;

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
    let model = cfg.sim.params.to_model_enum(dt);
    let model_copy = model.clone();
    let state = ndarray::Array::from(cfg.sim.state);

    let mut font = pango::FontDescription::new();
    font.set_family("Archivo Black");
    font.set_absolute_size(30.0 * f64::from(pango::SCALE));
    let _font = font.to_utilfont();

    let mut gui = sensoreval_gui::Context::default();
    gui.set_callback(Some(GuiCallback::new(model, state, move |cr, state| {
        // clone state so the lock can stay unlocked while drawing
        let state = state.read().unwrap().clone();
        model_copy.draw_state(cr, &state);
    })));

    gui.set_timer_ms(15);
    gui.start().unwrap();
}
