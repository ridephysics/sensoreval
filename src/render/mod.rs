pub mod utils;

use crate::*;
use approx::abs_diff_ne;

/// data source type and info
#[allow(clippy::large_enum_variant)]
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
    /// the timestamp that was actually requested
    pub actual_ts: u64,
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
pub struct Context<'a, 'b, 'c> {
    pub cfg: &'a config::Config,
    hudrenderer: Option<Box<dyn HudRenderer>>,
    hudctx: HudContext<'b>,
    blenderdir: Option<&'c std::path::Path>,
    videosz: Option<(usize, usize)>,
    allow_missing_renders: bool,
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
    /// get current orientation of the person sitting in the ride
    fn orientation(&self, ctx: &render::HudContext)
        -> Result<nalgebra::UnitQuaternion<f64>, Error>;
    /// serialize state into webdata
    fn serialize_forweb(
        &self,
        ctx: &render::HudContext,
        cfg: &config::Config,
        path: &std::path::Path,
    ) -> Result<(), Error>;
}

/// create a new HUD renderer
fn renderer_from_ctx(ctx: &Context) -> Option<Box<dyn HudRenderer>> {
    match &ctx.cfg.hud.renderer {
        config::HudRenderer::Pendulum(cfg) => Some(Box::new(
            hudrenderers::pendulum::Pendulum::new(&ctx.hudctx, cfg),
        )),
        config::HudRenderer::Generic => {
            Some(Box::new(hudrenderers::generic::Generic::new(&ctx.hudctx)))
        }
    }
}

impl<'a, 'b, 'c> Context<'a, 'b, 'c> {
    pub fn new(cfg: &'a config::Config, dataset: Option<&'b Vec<Data>>) -> Self {
        let mut ctx = Self {
            cfg,
            hudrenderer: None,
            hudctx: HudContext {
                dataset: None,
                dpi: 141.21,
                spi: 141.21,
                src: DataSrc::None,
                actual_ts: 0,
            },
            blenderdir: None,
            videosz: None,
            allow_missing_renders: false,
        };

        ctx.hudrenderer = renderer_from_ctx(&ctx);

        ctx.set_dataset(dataset);

        ctx
    }

    pub fn set_blenderdir(&mut self, blenderdir: Option<&'c std::path::Path>) {
        self.blenderdir = blenderdir;
    }

    pub fn set_videosz(&mut self, videosz: Option<(usize, usize)>) {
        self.videosz = videosz;
    }

    pub fn set_allow_missing_renders(&mut self, allow_missing_renders: bool) {
        self.allow_missing_renders = allow_missing_renders;
    }

    /// set timestamp in dataset, if available
    pub fn set_ts(&mut self, us: u64) -> Result<(), Error> {
        if self.hudctx.dataset.is_none() {
            return Err(Error::NoDataSet);
        }

        match id_for_time(self.hudctx.dataset.unwrap(), 0, us) {
            Some(id) => {
                self.hudctx.src = DataSrc::Array { id };
                self.hudctx.actual_ts = us;
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

            if let Some(blenderdir) = &self.blenderdir {
                cr.save();

                // scale blender graphics if the cairo surface size doesn't match the video resolution
                if let Some(videosz) = self.videosz {
                    if videosz != (ssz.0 as usize, ssz.1 as usize) {
                        cr.scale(
                            1.0 * ssz.0 / videosz.0 as f64,
                            1.0 * ssz.1 / videosz.1 as f64,
                        );
                    }
                }
                let ssz = render::utils::surface_sz_user(cr);

                let q = self.orientation()?;
                let fid = quat_to_fid(&q);
                let path = blenderdir.join(format!("mannequin/mannequin_{}.png", fid));

                if let Ok(surface) = utils::png_to_surface(&path) {
                    cr.set_source_surface(&surface, 0.0, ssz.1 - surface.get_height() as f64);
                    cr.paint();
                } else if !self.allow_missing_renders {
                    return Err(Error::BlenderRenderNotFound);
                }

                cr.restore();
            }
        }

        Ok(())
    }

    pub fn plot(&self) -> Result<(), Error> {
        if let Some(renderer) = &self.hudrenderer {
            renderer.plot(&self.hudctx)
        } else {
            Err(Error::NoHudRenderer)
        }
    }

    pub fn orientation(&self) -> Result<nalgebra::UnitQuaternion<f64>, Error> {
        if let Some(renderer) = &self.hudrenderer {
            renderer.orientation(&self.hudctx)
        } else {
            Err(Error::NoHudRenderer)
        }
    }

    pub fn serialize_forweb(&self, path: &std::path::Path) -> Result<(), Error> {
        if let Some(renderer) = &self.hudrenderer {
            renderer.serialize_forweb(&self.hudctx, &self.cfg, &path)
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

fn process_number_for_name(n: f64) -> f64 {
    let n = (n * 1000.0) as i64;
    match n.cmp(&0) {
        std::cmp::Ordering::Equal => 0.0,
        std::cmp::Ordering::Greater | std::cmp::Ordering::Less => (n as f64) / 1000.0,
    }
}

pub fn process_quat_for_name(q: &nalgebra::Vector4<f64>) -> nalgebra::Vector4<f64> {
    nalgebra::Vector4::new(
        process_number_for_name(q[0]),
        process_number_for_name(q[1]),
        process_number_for_name(q[2]),
        process_number_for_name(q[3]),
    )
}

pub fn quat_to_fid(q: &nalgebra::Quaternion<f64>) -> String {
    let q = process_quat_for_name(q.as_vector());
    format!("{:.3}_{:.3}_{:.3}_{:.3}", q[3], q[0], q[1], q[2])
}
