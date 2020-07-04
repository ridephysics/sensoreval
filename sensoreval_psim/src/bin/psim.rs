use sensoreval_psim::Model;

struct GuiCallback {
    state: std::sync::Arc<std::sync::RwLock<ndarray::Array1<f64>>>,
}

impl GuiCallback {
    pub fn new<M: 'static + Model + Send + Sync>(
        mut model: M,
        initial: ndarray::Array1<f64>,
    ) -> Self {
        let o = Self {
            state: std::sync::Arc::new(std::sync::RwLock::new(initial)),
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

impl sensoreval_gui::Callback for GuiCallback {
    fn render(&mut self, _ctx: &mut sensoreval_gui::RuntimeContext, cr: &mut cairo::Context) {
        let m1a = self.state.read().unwrap()[0];
        sensoreval_graphics::pendulum_2d::draw(cr, m1a);
    }
}

fn main() {
    let mut gui = sensoreval_gui::Context::default();
    gui.set_timer_ms(15);
    gui.set_callback(Some(GuiCallback::new(
        sensoreval_psim::models::Pendulum::new(1.0, 0.001),
        ndarray::array![std::f64::consts::PI - 0.1, 0.0],
    )));
    gui.start().unwrap();
}
