use crate::*;

#[no_mangle]
pub extern "C" fn sensoreval_render(
    renderctx_ptr: *mut render::Context,
    cr_ptr: *mut cairo_sys::cairo_t,
) -> std::os::raw::c_int {
    let renderctx = unwrap_opt_or!(unsafe { renderctx_ptr.as_mut() }, return -1);
    let cr = unsafe { cairo::Context::from_raw_borrow(cr_ptr) };

    unwrap_res_or!(renderctx.render(&cr), return -2);

    0
}

#[no_mangle]
pub extern "C" fn sensoreval_render_set_ts(
    renderctx_ptr: *mut render::Context,
    us: u64,
) -> std::os::raw::c_int {
    let renderctx = unwrap_opt_or!(unsafe { renderctx_ptr.as_mut() }, return -1);

    unwrap_res_or!(renderctx.set_ts(us), return -3);

    0
}

#[no_mangle]
pub extern "C" fn sensoreval_notify_stdin(
    renderctx_ptr: *mut render::Context,
    readctx_ptr: *mut datareader::Context,
) -> std::os::raw::c_int {
    let renderctx = unwrap_opt_or!(unsafe { renderctx_ptr.as_mut() }, return -1);
    let readctx = unwrap_opt_or!(unsafe { readctx_ptr.as_mut() }, return -2);
    let datacfg = if let config::DataSource::SensorData(sd) = &renderctx.cfg.data.source {
        sd
    } else {
        return -3;
    };

    let sample = match readctx.read_sample(&mut std::io::stdin(), datacfg) {
        Err(e) => match e {
            Error::EOF => return -4,
            Error::Io(eio) => match eio.kind() {
                std::io::ErrorKind::WouldBlock => {
                    return 0;
                }
                _ => return -5,
            },
            _ => return -6,
        },
        Ok(v) => v,
    };

    renderctx.set_data(sample);

    1
}

#[no_mangle]
pub extern "C" fn sensoreval_render_get_quat(
    renderctx_ptr: *const render::Context,
    quat_ptr: *mut std::os::raw::c_double,
) -> std::os::raw::c_int {
    let renderctx = unwrap_opt_or!(unsafe { renderctx_ptr.as_ref() }, return -1);
    let quat = unsafe { std::slice::from_raw_parts_mut(quat_ptr, 4) };
    let q = unwrap_res_or!(renderctx.orientation(), return -2);

    quat[0] = q[3] as std::os::raw::c_double;
    quat[1] = q[0] as std::os::raw::c_double;
    quat[2] = q[1] as std::os::raw::c_double;
    quat[3] = q[2] as std::os::raw::c_double;

    0
}

#[no_mangle]
pub extern "C" fn sensoreval_render_get_video_info(
    renderctx_ptr: *const render::Context,
    filename_ptr: *mut std::os::raw::c_char,
    filename_sz_uint: std::os::raw::c_uint,
    video_startoff_ptr: *mut u64,
    video_endoff_ptr: *mut u64,
) -> std::os::raw::c_int {
    let renderctx = unwrap_opt_or!(unsafe { renderctx_ptr.as_ref() }, return -1);
    let filename_sz = filename_sz_uint as usize;

    match &renderctx.cfg.video.filename {
        Some(v) => {
            if filename_sz <= v.len() {
                return -2;
            }
            unsafe {
                std::ptr::copy(v.as_ptr(), filename_ptr as *mut u8, v.len());
                *filename_ptr.add(v.len()) = 0;
            }
        }
        None => {
            if (filename_sz) < 1 {
                return -2;
            }
            unsafe {
                *filename_ptr = 0;
            }
        }
    }

    unsafe {
        *video_startoff_ptr = renderctx.cfg.video.startoff;
        *video_endoff_ptr = match renderctx.cfg.video.endoff {
            Some(v) => v,
            None => std::u64::MAX,
        }
    }

    0
}
