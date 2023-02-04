use crate::utils;
use crate::utils::CairoEx;

/// draws to pointy end of a boat
fn swingboat_head_line(cr: &cairo::Context, radius0: f64, angle0: f64, radius1: f64, angle1: f64) {
    let thickness = (radius0 - radius1).abs();
    let (angle_inner, angle_outer) = if radius1 <= radius0 {
        (angle0, angle1)
    } else {
        (angle1, angle0)
    };
    let mirror = if angle_outer <= angle_inner {
        1.0
    } else {
        -1.0
    };
    let (x0, y0) = utils::circle_coords(radius0, angle0);
    let (x1, y1) = utils::circle_coords(radius1, angle1);

    // tip
    let (tip_x, tip_y) = utils::circle_coords(
        thickness,
        angle_outer + (std::f64::consts::FRAC_PI_2 * mirror),
    );

    // bottom
    let (bottom_x, bottom_y) = utils::circle_coords(
        thickness / 2.0,
        angle_inner - (std::f64::consts::FRAC_PI_2 * mirror),
    );

    if radius1 <= radius0 {
        cr.curve_to(x0 + bottom_x, y0 + bottom_y, x1 + tip_x, y1 + tip_y, x1, y1);
    } else {
        cr.curve_to(x0 + tip_x, y0 + tip_y, x1 + bottom_x, y1 + bottom_y, x1, y1);
    }
}

pub fn draw(cr: &cairo::Context, gondola_rotation: f64, active_row: usize) {
    let frame_radius: f64 = 16.0;
    let frame_angle: f64 = (68.0f64).to_radians();
    let frame_thickness: f64 = 0.5;
    let frame_color: u32 = 0xffff_ffe5;
    let frame_top_color: u32 = 0x0000_00e5;
    let gondola_radius: f64 = 15.0;
    let gondola_angle: f64 = (30.0f64).to_radians();
    let gondola_frame_thickness: f64 = 0.4;
    let gondola_thickness: f64 = 1.8;
    let gondola_color: u32 = 0xff8f_00e5;
    let gondola_num_sections: usize = 5;
    let active_row_color: u32 = 0x64dd_17e5;
    let section_divider_color: u32 = 0x0000_00e5;
    let section_divider_width: f64 = 0.2;
    let section_dark_color: u32 = 0x0000_0033;
    let border_width: f64 = 0.1;
    let border_color: u32 = 0x0000_00ff;

    let (cx, cy) = cr.current_point().unwrap();

    cr.save().unwrap();
    cr.translate(cx, cy);

    cr.save().unwrap();
    cr.rotate(gondola_rotation);

    // gondola-frame
    cr.set_source_rgba_u32(frame_color);
    cr.set_line_width(gondola_frame_thickness);
    cr.move_to(0., 0.);
    // we use twice the radius because we want to cut the line horizontally
    // which is done by clipping it
    cr.stroke_arc_sides(
        gondola_radius,
        std::f64::consts::PI / 2.0,
        gondola_angle / 2.0
            + math::tri_solve_sas(
                frame_thickness / 2.0,
                gondola_radius,
                std::f64::consts::FRAC_PI_2,
            )
            .0,
        border_width,
        border_color,
    );

    // gondola
    {
        // the boat itself
        cr.save().unwrap();

        let angle_middle = std::f64::consts::FRAC_PI_2;
        let angle_left = angle_middle + gondola_angle / 2.0;
        let angle_right = angle_middle - gondola_angle / 2.0;
        let section_width_ang = gondola_angle / gondola_num_sections as f64;
        let angle_head_right = angle_right - section_width_ang;
        let angle_head_left = angle_left + section_width_ang;
        let radius_ship_outer = gondola_radius + gondola_thickness / 2.0;
        let radius_ship_inner = gondola_radius - gondola_thickness / 2.0;
        let gondola_line_width = gondola_thickness / 6.0;

        cr.set_operator(cairo::Operator::Source);
        cr.set_line_join(cairo::LineJoin::Round);

        let (x0, y0) = utils::circle_coords(radius_ship_outer, angle_right);
        cr.move_to(x0, y0);

        swingboat_head_line(
            cr,
            radius_ship_outer,
            angle_right,
            radius_ship_inner,
            angle_head_right,
        );
        cr.arc(
            0.0,
            0.0,
            radius_ship_inner,
            angle_head_right,
            angle_head_left,
        );
        swingboat_head_line(
            cr,
            radius_ship_inner,
            angle_head_left,
            radius_ship_outer,
            angle_left,
        );
        cr.arc_negative(0.0, 0.0, radius_ship_outer, angle_left, angle_right);
        cr.close_path();

        cr.set_line_width(gondola_line_width + border_width);
        cr.set_source_rgba_u32(border_color);
        cr.stroke_preserve().unwrap();

        cr.set_source_rgba_u32(gondola_color);
        cr.set_line_width(gondola_line_width);
        cr.stroke_preserve().unwrap();

        cr.fill().unwrap();
        cr.restore().unwrap();

        // sections
        cr.save().unwrap();
        cr.set_source_rgba_u32(section_dark_color);
        cr.set_line_width(gondola_thickness + gondola_line_width);

        // left
        cr.arc(
            0.0,
            0.0,
            gondola_radius,
            angle_left - section_width_ang,
            angle_left,
        );
        cr.stroke().unwrap();

        // middle
        cr.arc(
            0.0,
            0.0,
            gondola_radius,
            angle_middle - section_width_ang / 2.0,
            angle_middle + section_width_ang / 2.0,
        );
        cr.stroke().unwrap();

        // right
        cr.arc(
            0.0,
            0.0,
            gondola_radius,
            angle_right,
            angle_right + section_width_ang,
        );
        cr.stroke().unwrap();

        // active row
        let active_row_left = angle_left - active_row as f64 * section_width_ang / 2.0;
        cr.set_operator(cairo::Operator::Source);
        cr.set_source_rgba_u32(active_row_color);
        cr.arc_negative(
            0.0,
            0.0,
            gondola_radius,
            active_row_left,
            active_row_left - section_width_ang / 2.0,
        );
        cr.stroke().unwrap();

        cr.restore().unwrap();

        // section dividers
        let radius_divider_inner = radius_ship_inner - gondola_line_width / 2.0;
        let radius_divider_outer = radius_ship_outer + gondola_line_width / 2.0;

        cr.save().unwrap();
        cr.set_line_width(section_divider_width);

        for i in 0..gondola_num_sections {
            let angle = angle_left - section_width_ang / 2.0 - i as f64 * section_width_ang;

            cr.move_to_circle(radius_divider_inner, angle);
            cr.line_to_circle(radius_divider_outer, angle);

            cr.set_source_rgba_u32(section_divider_color);
            cr.set_operator(cairo::Operator::Source);
            cr.stroke().unwrap();
        }

        cr.restore().unwrap();
    }

    cr.restore().unwrap();

    // frame
    cr.save().unwrap();
    cr.clip_bottom(frame_radius);
    cr.set_source_rgba_u32(frame_color);
    cr.set_line_width(frame_thickness);
    cr.move_to(0., 0.);
    cr.stroke_arc_sides(
        frame_radius * 2.0,
        std::f64::consts::PI / 2.0,
        frame_angle / 2.0,
        border_width,
        border_color,
    );
    cr.restore().unwrap();

    // top
    cr.set_operator(cairo::Operator::Source);
    cr.set_source_rgba_u32(frame_color);
    cr.set_line_width(0.2);
    cr.arc(0., 0., 1.0, 0., 2.0 * std::f64::consts::PI);
    cr.fill_preserve().unwrap();
    cr.set_source_rgba_u32(frame_top_color);
    cr.stroke().unwrap();

    cr.restore().unwrap();
}
