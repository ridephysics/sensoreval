use crate::error::*;

use nalgebra::base::Vector3;

pub(crate) struct Data {
    angle: f64,
}

pub(crate) struct SwingBoat {
    ppm: f64,
    dataset: Vec<Data>,
}

impl SwingBoat {
    pub fn new(ctx: &crate::render::Context) -> Self {
        let mut sb = Self {
            ppm: 1.,
            dataset: Vec::new(),
        };

        match ctx.dataset {
            Some(dataset) => {
                for data in dataset {
                    let vnorth = Vector3::new(0., 1., 0.);
                    let mut vnorthrot = data.quat * vnorth;

                    vnorthrot[0] = 0.;
                    vnorthrot[1] = (1.0 - vnorthrot[2].powf(2.0)).sqrt();

                    let mut angle = vnorth.angle(&vnorthrot);
                    if vnorthrot[2] < 0. {
                        angle *= -1.;
                    }

                    sb.dataset.push(Data { angle: angle });
                }
            }
            None => {}
        }

        return sb;
    }
}

impl crate::render::HudHandler for SwingBoat {
    fn render(&self, ctx: &crate::render::Context, cr: &cairo::Context) -> Result<(), Error> {
        return Ok(());
    }
}
