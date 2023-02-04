use crate::utils::CairoEx;

pub fn draw(cr: &cairo::Context, m1a: f64, m2a: f64, l1: f64, l2: f64) {
    let ssz = cr.surface_sz_user();
    let m1r = 40.0;
    let m2r = 40.0;
    let len_total = ssz.1 / 2.0 - m2r;
    let m1y = len_total / (l1 + l2) * l1;
    let m2y = len_total / (l1 + l2) * l2;

    cr.save().unwrap();
    cr.translate(ssz.0 / 2.0, ssz.1 / 2.0);
    cr.rotate(m1a);

    // rod1
    cr.set_source_rgba_u32(0xffffffff);
    cr.set_line_width(10.0);
    cr.move_to(0.0, 0.0);
    cr.line_to(0.0, m1y);
    cr.stroke().unwrap();

    // m1
    cr.save().unwrap();
    cr.translate(0.0, m1y);
    cr.set_source_rgba_u32(0xffffffff);
    cr.arc(0.0, 0.0, m1r, 0.0, 2.0 * std::f64::consts::PI);
    cr.fill().unwrap();
    cr.restore().unwrap();

    cr.translate(0.0, m1y);
    cr.rotate(-m1a);
    cr.rotate(m2a);

    // rod2
    cr.set_source_rgba_u32(0xffffffff);
    cr.set_line_width(10.0);
    cr.move_to(0.0, 0.0);
    cr.line_to(0.0, m2y);
    cr.stroke().unwrap();

    // m2
    cr.save().unwrap();
    cr.translate(0.0, m2y);
    cr.set_source_rgba_u32(0xffffffff);
    cr.arc(0.0, 0.0, m2r, 0.0, 2.0 * std::f64::consts::PI);
    cr.fill().unwrap();
    cr.restore().unwrap();

    cr.restore().unwrap();
}
