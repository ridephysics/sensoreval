pub mod utils;

use crate::*;
use approx::abs_diff_ne;

/// data source type and info
#[derive(Debug)]
enum DataSrc {
    /// no data available
    None,
    /// single sample source, e.g. from live data
    Data(Data),
    /// full dataset, currently set to the given index
    Array { id: usize },
}

/// context ṕassed to hud renderers
#[derive(Debug)]
pub struct HudContext<'b> {
    /// current dataset
    dataset: Option<&'b Vec<Data>>,
    /// data source type
    src: DataSrc,
    /// DPI, used for rendering graphics
    pub dpi: f64,
    /// SPI, used for rendering  text
    pub spi: f64,
}

impl<'b> HudContext<'b> {
    /// return current data sample, if any
    pub fn current_data(&self) -> Option<&Data> {
        match &self.src {
            DataSrc::None => None,
            DataSrc::Data(data) => Some(&data),
            DataSrc::Array { id } => match self.dataset {
                None => None,
                Some(arr) => Some(&arr[*id]),
            },
        }
    }

    /// return current dataset index, if available
    pub fn current_data_id(&self) -> Option<usize> {
        match &self.src {
            DataSrc::None => None,
            DataSrc::Data(_) => None,
            DataSrc::Array { id } => match self.dataset {
                None => None,
                Some(_) => Some(*id),
            },
        }
    }

    /// get dataset, if available
    pub fn get_dataset(&self) -> Option<&'b Vec<Data>> {
        match &self.src {
            DataSrc::None => None,
            DataSrc::Data(_) => None,
            DataSrc::Array { .. } => match self.dataset {
                None => None,
                Some(dataset) => Some(dataset),
            },
        }
    }

    /// convert DPI to pixels
    #[inline]
    pub fn dp2px(&self, dp: f64) -> f64 {
        dp * (self.dpi / 160.0)
    }

    /// convert SPI to pixels
    #[inline]
    pub fn sp2px(&self, sp: f64) -> f64 {
        sp * (self.spi / 160.0)
    }
}

/// rendering context
pub struct Context<'a, 'b> {
    cfg: &'a config::Config,
    hudrenderer: Option<Box<dyn HudRenderer>>,
    pub hudctx: HudContext<'b>,
}

/// HUD renderer trait
pub trait HudRenderer {
    /// called when the data source has changed, also called directly after constructor
    fn data_changed(&mut self, ctx: &render::HudContext);
    /// called when dpi or spi has changed
    fn scale_changed(&mut self, ctx: &render::HudContext);
    /// render current state to cairo context
    fn render(&self, ctx: &render::HudContext, cr: &cairo::Context) -> Result<(), Error>;
    /// plot dataset using matplotlib, if available
    fn plot(&self, ctx: &render::HudContext) -> Result<(), Error>;
}

/// create a new HUD renderer
fn renderer_from_ctx(ctx: &Context) -> Option<Box<dyn HudRenderer>> {
    let renderer = match &ctx.cfg.hud.renderer {
        config::HudRenderer::Pendulum(cfg) => {
            hudrenderers::pendulum::Pendulum::new(&ctx.hudctx, cfg)
        }
        _ => return None,
    };

    Some(Box::new(renderer))
}

impl<'a, 'b> Context<'a, 'b> {
    pub fn new(cfg: &'a config::Config, dataset: Option<&'b Vec<Data>>) -> Self {
        let mut ctx = Self {
            cfg,
            hudrenderer: None,
            hudctx: HudContext {
                dataset: None,
                dpi: 141.21,
                spi: 141.21,
                src: DataSrc::None,
            },
        };

        ctx.hudrenderer = renderer_from_ctx(&ctx);

        ctx.set_dataset(dataset);

        ctx
    }

    /// set timestamp in dataset, if available
    pub fn set_ts(&mut self, us: u64) -> Result<(), Error> {
        if self.hudctx.dataset.is_none() {
            return Err(Error::NoDataSet);
        }

        match id_for_time(self.hudctx.dataset.unwrap(), 0, us) {
            Some(id) => {
                self.hudctx.src = DataSrc::Array { id };
                Ok(())
            }
            None => Err(Error::SampleNotFound),
        }
    }

    /// use a single sample as the data source
    pub fn set_data(&mut self, data: Data) {
        self.hudctx.src = DataSrc::Data(data);
        self.hudctx.dataset = None;

        if let Some(renderer) = &mut self.hudrenderer {
            renderer.data_changed(&self.hudctx);
        }
    }

    /// use a dataset as the data source
    pub fn set_dataset(&mut self, dataset: Option<&'b Vec<Data>>) {
        self.hudctx.src = DataSrc::Array { id: 0 };
        self.hudctx.dataset = dataset;

        if let Some(renderer) = &mut self.hudrenderer {
            renderer.data_changed(&self.hudctx);
        }
    }

    pub fn current_data(&self) -> Option<&Data> {
        self.hudctx.current_data()
    }

    pub fn current_data_id(&self) -> Option<usize> {
        self.hudctx.current_data_id()
    }

    pub fn get_dataset(&self) -> Option<&'b Vec<Data>> {
        self.hudctx.get_dataset()
    }

    pub fn render(&mut self, cr: &cairo::Context) -> Result<(), Error> {
        let ssz = render::utils::surface_sz_user(cr);
        let dpi = 160.0 * (ssz.0 / 1920.0);
        let spi = dpi;
        let scale_changed =
            abs_diff_ne!(self.hudctx.dpi, dpi) || abs_diff_ne!(self.hudctx.spi, spi);
        self.hudctx.dpi = dpi;
        self.hudctx.spi = dpi;

        // clear
        cr.save();
        cr.set_source_rgba(0., 0., 0., 0.);
        cr.set_operator(cairo::Operator::Source);
        cr.paint();
        cr.restore();

        if let Some(renderer) = &mut self.hudrenderer {
            if scale_changed {
                renderer.scale_changed(&self.hudctx);
            }

            cr.save();
            let hudret = renderer.render(&self.hudctx, cr);
            cr.restore();
            hudret?;
        }

        Ok(())
    }

    /*
    extern "C" fn render_hud_cb(
        context: *mut ::std::os::raw::c_void,
        buf: *mut u8,
        w: crate::bindings::size_t,
        h: crate::bindings::size_t,
    ) -> ::std::os::raw::c_int {
        let renderctx = unsafe { (context as *mut Self).as_mut().unwrap() };

        let surface = cairo::ImageSurface::create_for_data(
            unsafe { std::slice::from_raw_parts_mut(buf, (w * h * 4) as usize) },
            cairo::Format::ARgb32,
            w as i32,
            h as i32,
            w as i32 * 4,
        )
        .expect("Can't create surface");
        let cr = cairo::Context::new(&surface);
        cr.set_antialias(cairo::Antialias::Best);
        renderctx.render(&cr).expect("can't render");
        surface.flush();

        0
    }

    pub fn render_video(&mut self, dst: &str) -> Result<(), Error> {
        let dst = std::ffi::CString::new(dst)?;
        let src = std::ffi::CString::new(
            self.cfg
                .video
                .filename
                .as_ref()
                .ok_or(Error::NoVideoFile)?
                .as_str(),
        )?;
        let rc = unsafe {
            crate::bindings::sensoreval_render_native_render_video(
                dst.as_ptr(),
                src.as_ptr(),
                Some(Self::render_hud_cb),
                self as *mut Self as *mut ::std::os::raw::c_void,
            )
        };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::from(rc))
        }
    }*/

    pub fn plot(&self) -> Result<(), Error> {
        if let Some(renderer) = &self.hudrenderer {
            renderer.plot(&self.hudctx)
        } else {
            Err(Error::NoHudRenderer)
        }
    }

    #[inline]
    pub fn dp2px(&self, dp: f64) -> f64 {
        self.hudctx.dp2px(dp)
    }

    #[inline]
    pub fn sp2px(&self, sp: f64) -> f64 {
        self.hudctx.sp2px(sp)
    }
}
