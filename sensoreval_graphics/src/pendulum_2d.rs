use crate::utils::CairoEx;

pub fn draw(cr: &cairo::Context, m1a: f64) {
    let ssz = cr.surface_sz_user();
    let m1r = 40.0;
    let m1y = ssz.1 / 2.0 - m1r;

    cr.save().unwrap();
    cr.translate(ssz.0 / 2.0, ssz.1 / 2.0);
    cr.rotate(-m1a);

    // rod
    cr.set_line_width(10.0);
    cr.move_to(0.0, 0.0);
    cr.line_to(0.0, m1y);
    cr.stroke().unwrap();

    // m1
    cr.save().unwrap();
    cr.translate(0.0, m1y);
    cr.arc(0.0, 0.0, m1r, 0.0, 2.0 * std::f64::consts::PI);
    cr.fill().unwrap();
    cr.restore().unwrap();

    cr.restore().unwrap();
}
