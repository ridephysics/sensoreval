use crate::*;

use std::convert::TryFrom;
use std::convert::TryInto;

pub struct Font<'a> {
    pgfont: &'a pango::FontDescription,
    pub color_fill: u32,
    pub color_border: u32,
    pub line_width: f64,
}

impl<'a> Font<'a> {
    pub fn new(pgfont: &'a pango::FontDescription) -> Self {
        Self {
            pgfont,
            color_fill: 0xffff_ffff,
            color_border: 0x0000_00ff,
            line_width: 1.0,
        }
    }

    pub fn layout(&self, cr: &cairo::Context, text: &str) -> pango::Layout {
        let layout = pangocairo::functions::create_layout(cr).unwrap();
        layout.set_font_description(Some(self.pgfont));
        layout.set_text(text);

        layout
    }

    pub fn draw_layout(&self, cr: &cairo::Context, layout: &pango::Layout) {
        cr.save();

        set_source_rgba_u32(cr, self.color_fill);
        pangocairo::functions::update_layout(cr, &layout);
        pangocairo::functions::show_layout(cr, &layout);

        cr.set_line_width(self.line_width);
        set_source_rgba_u32(cr, self.color_border);
        pangocairo::functions::layout_path(cr, &layout);
        cr.stroke();

        cr.restore();
    }

    pub fn draw(&self, cr: &cairo::Context, text: &str) -> (i32, i32) {
        let layout = self.layout(cr, text);
        self.draw_layout(cr, &layout);

        layout.get_pixel_size()
    }
}

pub fn set_source_rgba_u32(cr: &cairo::Context, rgba: u32) {
    let r: f64 = ((rgba >> 24) & 0xff).try_into().unwrap();
    let g: f64 = ((rgba >> 16) & 0xff).try_into().unwrap();
    let b: f64 = ((rgba >> 8) & 0xff).try_into().unwrap();
    let a: f64 = (rgba & 0xff).try_into().unwrap();

    let rf = 1.0 / 255.0 * r;
    let gf = 1.0 / 255.0 * g;
    let bf = 1.0 / 255.0 * b;
    let af = 1.0 / 255.0 * a;

    cr.set_source_rgba(rf, gf, bf, af);
}

pub fn surface_sz_user(cr: &cairo::Context) -> (f64, f64) {
    let surface = cairo::ImageSurface::try_from(cr.get_target()).unwrap();
    let sw = f64::from(surface.get_width());
    let sh = f64::from(surface.get_height());

    cr.device_to_user_distance(sw, sh)
}

#[allow(clippy::too_many_arguments)]
pub fn draw_graph<T: Iterator<Item = u64>, D: Iterator<Item = f64>>(
    cr: &cairo::Context,
    iter_time: &mut T,
    iter_data: &mut D,
    graph_width: f64,
    graph_height: f64,
    dt: u64,
    maxval: f64,
    redval: f64,
) {
    cr.save();
    cr.rectangle(0.0, 0.0, graph_width, graph_height);
    cr.clip();

    // graph-line style
    cr.set_line_width(6.0);
    let pat = cairo::LinearGradient::new(
        0.0,
        graph_height - (graph_height / maxval * redval),
        0.0,
        graph_height,
    );
    pat.add_color_stop_rgb(0.0, 1.0, 0.0, 0.0);
    pat.add_color_stop_rgb(0.5, 1.0, 1.0, 0.0);
    pat.add_color_stop_rgb(1.0, 0.0, 1.0, 0.0);
    cr.set_source(&pat);

    let mut tnow: u64 = 0;
    let mut tstart: u64 = 0;
    let mut first: bool = true;

    // clippy is wrong, because we're iterating of two iterators at the same time
    // alos, I'm not comfortable using zip, yet due to unclear performance impacts
    #[allow(clippy::while_let_loop)]
    loop {
        let time = unwrap_opt_or!(iter_time.next(), break);
        let data = unwrap_opt_or!(iter_data.next(), break);
        let was_first = first;
        if first {
            tnow = time;
            tstart = match time.checked_sub(dt) {
                None => 0,
                Some(v) => v,
            };
            first = false;
        }

        if time < tstart {
            break;
        }

        let x = graph_width - (graph_width / (dt as f64) * ((tnow - time) as f64));
        let y = graph_height - (graph_height / maxval * data);

        if was_first {
            cr.move_to(x, y);
        } else {
            cr.line_to(x, y);
        }
    }

    cr.stroke();
    cr.restore();

    // border
    render::utils::set_source_rgba_u32(cr, 0x0000_00ff);
    cr.set_line_width(3.0);
    cr.move_to(0.0, 0.0);
    cr.line_to(0.0, graph_height);
    cr.line_to(graph_width, graph_height);
    cr.stroke();
}
