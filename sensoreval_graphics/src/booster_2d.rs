use crate::utils::CairoEx;

pub fn draw(cr: &cairo::Context, m1a: f64, m2a: f64, m3a: f64) {
    let ssz = cr.surface_sz_user();
    let m1r = 10.0;
    let m1y = ssz.1 / 2.0 - m1r;

    cr.save();
    cr.translate(ssz.0 / 2.0, ssz.1 / 2.0);
    cr.rotate(m3a);

    // rod
    cr.set_line_width(10.0);
    cr.move_to(0.0, -200.0);
    cr.line_to(0.0, 200.0);
    cr.stroke();

    cr.set_line_width(5.0);

    // m1
    cr.set_source_rgba_u32(0xff0000ff);
    cr.save();
    cr.translate(0.0, 200.0);
    cr.rotate(-m3a);
    cr.rotate(m1a);
    cr.move_to(0.0, 0.0);
    cr.line_to(0.0, 50.0);
    cr.stroke();

    cr.translate(0.0, 50.0);
    cr.arc(0.0, 0.0, m1r, 0.0, 2.0 * std::f64::consts::PI);
    cr.fill();
    cr.restore();

    // m2
    cr.set_source_rgba_u32(0x00ff00ff);
    cr.save();
    cr.translate(0.0, -200.0);
    cr.rotate(-m3a);
    cr.rotate(m2a);
    cr.move_to(0.0, 0.0);
    cr.line_to(0.0, 50.0);
    cr.stroke();

    cr.translate(0.0, 50.0);
    cr.arc(0.0, 0.0, m1r, 0.0, 2.0 * std::f64::consts::PI);
    cr.fill();
    cr.restore();

    cr.restore();
}
