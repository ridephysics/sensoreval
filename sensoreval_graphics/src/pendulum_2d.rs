use crate::utils::CairoEx;

pub fn draw(cr: &cairo::Context, m1a: f64) {
    let ssz = cr.surface_sz_user();
    let m1r = 40.0;
    let m1y = ssz.1 / 2.0 - m1r;

    cr.save();
    cr.translate(ssz.0 / 2.0, ssz.1 / 2.0);
    cr.rotate(m1a);

    // clear
    cr.set_source_rgba_u32(0x00000000);
    cr.clear();

    // rod
    cr.set_source_rgba_u32(0xffffffff);
    cr.set_line_width(10.0);
    cr.move_to(0.0, 0.0);
    cr.line_to(0.0, m1y);
    cr.stroke();

    // m1
    cr.save();
    cr.translate(0.0, m1y);
    cr.set_source_rgba_u32(0xffffffff);
    cr.arc(0.0, 0.0, m1r, 0.0, 2.0 * std::f64::consts::PI);
    cr.fill();
    cr.restore();

    cr.restore();
}
