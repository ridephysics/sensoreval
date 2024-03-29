use crate::Error;
use std::convert::TryFrom;
use std::convert::TryInto;

pub trait CairoEx {
    fn clear(&self);
    fn set_source_rgba_u32(&self, rgba: u32);
    fn clip_bottom(&self, h: f64);
    fn move_to_circle(&self, r: f64, angle: f64);
    fn rel_move_to_circle(&self, r: f64, angle: f64);
    fn line_to_circle(&self, r: f64, angle: f64);
    fn stroke_arc_sides(
        &self,
        r: f64,
        angle_mid: f64,
        angle_sides: f64,
        border_width: f64,
        border_color: u32,
    );
    fn surface_sz_user(&self) -> (f64, f64);
}

impl CairoEx for cairo::Context {
    fn clear(&self) {
        self.save().unwrap();
        self.set_operator(cairo::Operator::Source);
        self.paint().unwrap();
        self.restore().unwrap();
    }

    fn set_source_rgba_u32(&self, rgba: u32) {
        let r: f64 = ((rgba >> 24) & 0xff).try_into().unwrap();
        let g: f64 = ((rgba >> 16) & 0xff).try_into().unwrap();
        let b: f64 = ((rgba >> 8) & 0xff).try_into().unwrap();
        let a: f64 = (rgba & 0xff).try_into().unwrap();

        let rf = 1.0 / 255.0 * r;
        let gf = 1.0 / 255.0 * g;
        let bf = 1.0 / 255.0 * b;
        let af = 1.0 / 255.0 * a;

        self.set_source_rgba(rf, gf, bf, af);
    }

    fn clip_bottom(&self, h: f64) {
        let ssz = self.surface_sz_user();
        let p = self.device_to_user(0., 0.).unwrap();
        let sz = (ssz.0, -p.1 + h);

        self.rectangle(p.0, p.1, sz.0, sz.1);
        self.clip();
    }

    fn move_to_circle(&self, r: f64, angle: f64) {
        let (x, y) = circle_coords(r, angle);
        self.move_to(x, y)
    }

    fn rel_move_to_circle(&self, r: f64, angle: f64) {
        let (x, y) = circle_coords(r, angle);
        self.rel_move_to(x, y)
    }

    fn line_to_circle(&self, r: f64, angle: f64) {
        let (x, y) = circle_coords(r, angle);
        self.line_to(x, y)
    }

    fn stroke_arc_sides(
        &self,
        r: f64,
        angle_mid: f64,
        angle_sides: f64,
        border_width: f64,
        border_color: u32,
    ) {
        let (cx, cy) = self.current_point().unwrap();

        self.save().unwrap();
        self.set_line_width(self.line_width() + border_width);
        self.set_source_rgba_u32(border_color);
        self.rel_move_to_circle(r, angle_mid + angle_sides);
        self.line_to(cx, cy);
        self.line_to_circle(r, angle_mid - angle_sides);
        self.stroke_preserve().unwrap();
        self.restore().unwrap();

        self.save().unwrap();
        self.set_operator(cairo::Operator::Source);
        self.stroke().unwrap();
        self.restore().unwrap();
    }

    fn surface_sz_user(&self) -> (f64, f64) {
        let surface = cairo::ImageSurface::try_from(self.target()).unwrap();
        let sw = f64::from(surface.width());
        let sh = f64::from(surface.height());

        self.device_to_user_distance(sw, sh).unwrap()
    }
}

pub fn circle_coords(r: f64, angle: f64) -> (f64, f64) {
    (angle.cos() * r, angle.sin() * r)
}

pub struct Font<'a> {
    pgfont: std::borrow::Cow<'a, pango::FontDescription>,
    pub color_fill: u32,
    pub color_border: u32,
    pub line_width: f64,
}

impl<'a> Font<'a> {
    pub fn new<F: Into<std::borrow::Cow<'a, pango::FontDescription>>>(pgfont: F) -> Self {
        Self {
            pgfont: pgfont.into(),
            color_fill: 0xffff_ffff,
            color_border: 0x0000_00ff,
            line_width: 1.0,
        }
    }

    pub fn layout(&self, cr: &cairo::Context, text: &str) -> pango::Layout {
        let layout = pangocairo::functions::create_layout(cr);
        layout.set_font_description(Some(self.pgfont.as_ref()));
        layout.set_text(text);

        layout
    }

    pub fn draw_layout(&self, cr: &cairo::Context, layout: &pango::Layout) {
        cr.save().unwrap();

        cr.set_source_rgba_u32(self.color_fill);
        pangocairo::functions::update_layout(cr, layout);
        pangocairo::functions::show_layout(cr, layout);

        cr.set_line_width(self.line_width);
        cr.set_source_rgba_u32(self.color_border);
        pangocairo::functions::layout_path(cr, layout);
        cr.stroke().unwrap();

        cr.restore().unwrap();
    }

    pub fn draw(&self, cr: &cairo::Context, text: &str) -> (i32, i32) {
        let layout = self.layout(cr, text);
        self.draw_layout(cr, &layout);

        layout.pixel_size()
    }
}

pub trait ToUtilFont {
    fn into_utilfont<'a>(self) -> Font<'a>;
    fn utilfont(&self) -> Font;
}

impl ToUtilFont for pango::FontDescription {
    fn into_utilfont<'a>(self) -> Font<'a> {
        Font::new(std::borrow::Cow::Owned(self))
    }

    fn utilfont(&self) -> Font {
        Font::new(std::borrow::Cow::Borrowed(self))
    }
}

pub struct Graph {
    pub width: f64,
    pub height: f64,
    pub dt: u64,
    pub maxval: f64,
    pub redval: f64,
    pub line_width: f64,
    pub border_color: u32,
    pub border_width: f64,
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            width: 2.0,
            height: 1.0,
            dt: 10_000_000,
            maxval: 1.0,
            redval: 2.0,
            line_width: 1.0,
            border_color: 0x0000_00ff,
            border_width: 1.0,
        }
    }
}

impl Graph {
    pub fn draw<T: Iterator<Item = u64>, D: Iterator<Item = f64>>(
        &self,
        cr: &cairo::Context,
        iter_time: &mut T,
        iter_data: &mut D,
    ) -> f64 {
        let (cx, cy) = cr.current_point().unwrap();

        cr.save().unwrap();
        cr.rectangle(cx, cy, self.width, self.height);
        cr.clip();

        // graph-line style
        cr.set_line_width(self.line_width);
        let pat = cairo::LinearGradient::new(
            0.0,
            self.height - (self.height / self.maxval * self.redval),
            0.0,
            self.height,
        );
        pat.add_color_stop_rgb(0.0, 1.0, 0.0, 0.0);
        pat.add_color_stop_rgb(0.5, 1.0, 1.0, 0.0);
        pat.add_color_stop_rgb(1.0, 0.0, 1.0, 0.0);
        cr.set_source(&pat).unwrap();

        let mut tnow: u64 = 0;
        let mut tstart: u64 = 0;
        let mut first: bool = true;
        let mut data_now: f64 = 0.0;
        for (time, data) in iter_time.zip(iter_data) {
            let was_first = first;
            if first {
                tnow = time;
                tstart = time.saturating_sub(self.dt);
                first = false;
                data_now = data;
            }

            if time < tstart {
                break;
            }

            let x = cx + self.width - (self.width / (self.dt as f64) * ((tnow - time) as f64));
            let y = cy + self.height - (self.height / self.maxval * data);

            if was_first {
                cr.move_to(x, y);
            } else {
                cr.line_to(x, y);
            }
        }

        cr.stroke().unwrap();
        cr.restore().unwrap();

        // border
        cr.save().unwrap();
        cr.set_source_rgba_u32(self.border_color);
        cr.set_line_width(self.border_width);
        cr.move_to(cx, cy);
        cr.rel_line_to(0.0, self.height);
        cr.rel_line_to(self.width, 0.0);
        cr.stroke().unwrap();
        cr.restore().unwrap();

        data_now
    }
}

pub struct GraphAndText<'a, 'b, 'c> {
    font: &'a Font<'a>,
    pub graph: Graph,
    pub unit: &'b str,
    pub precision: usize,
    pub graph_x: f64,
    pub icon: Option<&'c librsvg::SvgHandle>,
}

impl<'a, 'b, 'c> GraphAndText<'a, 'b, 'c> {
    pub fn new(font: &'a Font<'a>) -> Self {
        Self {
            font,
            graph: Graph::default(),
            unit: "",
            precision: 0,
            graph_x: 0.0,
            icon: None,
        }
    }

    pub fn draw<T: Iterator<Item = u64>, D: Iterator<Item = f64>>(
        &self,
        cr: &cairo::Context,
        iter_time: &mut T,
        iter_data: &mut D,
    ) {
        let (cx, cy) = cr.current_point().unwrap();

        cr.move_to(cx + self.graph_x, cy);
        let value_now = self.graph.draw(cr, iter_time, iter_data);

        // text
        let layout = self.font.layout(
            cr,
            &format!("{:.*}{}", self.precision, value_now, self.unit),
        );

        let mut current_x = cx;
        let current_y = cy;
        if let Some(icon) = self.icon {
            let svg_renderer = librsvg::CairoRenderer::new(icon);
            svg_renderer
                .render_document(
                    cr,
                    &cairo::Rectangle::new(
                        current_x,
                        current_y,
                        self.graph.height,
                        self.graph.height,
                    ),
                )
                .unwrap();

            current_x += self.graph.height;
        }

        cr.move_to(
            current_x,
            current_y + (self.graph.height - layout.pixel_size().1 as f64) / 2.0,
        );
        self.font.draw_layout(cr, &layout);
    }
}

pub fn bytes_to_svghandle(data: &'static [u8]) -> librsvg::SvgHandle {
    let bytes = glib::Bytes::from_static(data);
    let stream = gio::MemoryInputStream::from_bytes(&bytes);
    librsvg::Loader::new()
        .read_stream(&stream, None::<&gio::File>, None::<&gio::Cancellable>)
        .unwrap()
}

pub fn png_to_surface(path: &std::path::Path) -> Result<cairo::ImageSurface, Error> {
    let mut file = std::fs::File::open(path)?;
    Ok(cairo::ImageSurface::create_from_png(&mut file)?)
}
