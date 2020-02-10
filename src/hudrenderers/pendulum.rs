use crate::*;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

pub(crate) struct Pendulum {
    cfg: Config,
}

impl Pendulum {
    pub fn new(_ctx: &render::Context, cfg: &Config) -> Self {
        Self {
            cfg: (*cfg).clone(),
        }
    }

    #[inline]
    fn get_actual(data: &'_ Data) -> Option<&'_ simulator::pendulum::Actual> {
        if let data::ActualData::Pendulum(p) = data.actual.as_ref()?.as_ref() {
            return Some(p);
        }

        None
    }
}

impl render::HudRenderer for Pendulum {
    fn render(&self, _ctx: &render::Context, _cr: &cairo::Context) -> Result<(), Error> {
        Ok(())
    }

    fn plot(&self, ctx: &render::Context) -> Result<(), Error> {
        let samples = ctx.dataset.ok_or(Error::NoDataSet)?;

        let mut plot = Plot::new(
            "\
            import math\n\
            \n\
            ca = '#1f77b4'\n\
            cz = '#ff7f0e'\n\
            ce = '#2ca02c'\n\
            \n\
            t = np.array(pickle.load(sys.stdin.buffer))\n\
            z = np.array(pickle.load(sys.stdin.buffer))\n\
            has_actual = pickle.load(sys.stdin.buffer)\n\
            if has_actual:\n\
                \tactual = np.array(pickle.load(sys.stdin.buffer))\n\
            \n\
            fig, plots = plt.subplots(6, sharex=True)\n\
            \n\
            plots[0].set_title('p_a', x=-0.15, y=0.5)\n\
            if has_actual:\n\
                \tplots[0].plot(t, np.degrees(actual[:, 0]), ca)\n\
            \n\
            plots[1].set_title('v_a', x=-0.15, y=0.5)\n\
            plots[1].plot(t, np.degrees(z[:, 3]), cz)\n\
            if has_actual:\n\
                \tplots[1].plot(t, np.degrees(actual[:, 1]), ca)\n\
            \n\
            plots[2].set_title('v_t', x=-0.15, y=0.5)\n\
            if has_actual:\n\
                \tplots[2].plot(t, actual[:, 2], ca)\n\
            \n\
            plots[3].set_title('a_t', x=-0.15, y=0.5)\n\
            plots[3].plot(t, z[:, 2], cz)\n\
            if has_actual:\n\
                \tplots[3].plot(t, actual[:, 3], ca)\n\
            \n\
            plots[4].set_title('pa_a', x=-0.15, y=0.5)\n\
            \n\
            plots[5].set_title('va_a', x=-0.15, y=0.5)\n\
            \n\
            fig.tight_layout()\n\
            plt.show()\n\
            ",
        )?;

        plot.write(&DataSerializer::new(&samples, |_i, v| v.time_seconds()))?;

        plot.write(&DataSerializer::new(&samples, |_i, v| {
            vec![
                v.accel[0], v.accel[1], v.accel[2], v.gyro[0], v.gyro[1], v.gyro[2],
            ]
        }))?;

        let has_actual = if let Some(sample) = samples.first() {
            Self::get_actual(&sample).is_some()
        } else {
            false
        };
        plot.write(&has_actual)?;

        if has_actual {
            plot.write(&DataSerializer::new(&samples, |_i, v| {
                let actual = Self::get_actual(v).unwrap();
                vec![actual.p_ang, actual.v_ang, actual.v_tan, actual.a_tan]
            }))?;
        }

        plot.wait()
    }
}
