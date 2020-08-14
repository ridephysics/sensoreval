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
    let matches = clap::App::new("psim")
        .arg(
            clap::Arg::with_name("NAME")
                .help("name of the simulation")
                .required(true)
                .index(1),
        )
        .get_matches();
    let name = matches.value_of("NAME").unwrap();
    let mut gui = sensoreval_gui::Context::default();

    let mut font = pango::FontDescription::new();
    font.set_family("Archivo Black");
    font.set_absolute_size(30.0 * f64::from(pango::SCALE));
    let font = font.to_utilfont();

    match name {
        "pendulum" => {
            gui.set_callback(Some(GuiCallback::new(
                sensoreval_psim::models::Pendulum::new(1.0, 0.001),
                ndarray::array![std::f64::consts::PI - 0.1, 0.0],
                |cr, state| {
                    let m1a = state.read().unwrap()[0];
                    sensoreval_graphics::pendulum_2d::draw(cr, m1a);
                },
            )));
        }
        _ => panic!("unsupported sim {}", name),
    }

    gui.set_timer_ms(15);
    gui.start().unwrap();
}
