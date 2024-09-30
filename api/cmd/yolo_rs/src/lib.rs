#[cfg(test)]
mod tests;

mod yolo;

use image::io::Reader as ImageReader;
use std::ffi::{CStr, CString};
use std::io::Cursor;
use std::os::raw::c_char;
use std::ptr;
use std::slice;
pub use yolo::{YOLOv8, YOLOv8Config};

#[no_mangle]
pub extern "C" fn load_model(
    c_model_path: *const c_char,
    c_save_dir: *const c_char,
) -> *mut YOLOv8 {
    if c_model_path.is_null() {
        return ptr::null_mut();
    };

    let c_string_model_path = unsafe { CString::from(CStr::from_ptr(c_model_path)) };

    let model_path = match c_string_model_path.into_string() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let dir = match c_save_dir.is_null() {
        true => "model_results",
        false => {
            let c_str_save_dir = unsafe { CStr::from_ptr(c_save_dir) };

            match c_str_save_dir.to_str() {
                Ok(v) => v,
                Err(_) => "model_results",
            }
        }
    };

    let model = match YOLOv8::new(YOLOv8Config {
        model_path,
        conf: 0.55,
        profile: false,
        plot: true,
        save_dir: Some(dir.to_string()),
    }) {
        Ok(v) => v,
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(model))
}

#[no_mangle]
pub extern "C" fn free_model(c_model: *mut YOLOv8) {
    if !c_model.is_null() {
        unsafe {
            let _ = Box::from_raw(c_model);
        }
    }
}

#[no_mangle]
pub extern "C" fn process_image(
    c_model: *mut YOLOv8,
    buffer: *const u8,
    length: i32,
) -> *const c_char {
    let model = match c_model.is_null() {
        false => unsafe { &mut *c_model },
        true => {
            println!("Invalid pointer to model");
            return ptr::null();
        }
    };

    if buffer.is_null() || length <= 0 {
        println!("Invalid buffer");
        return ptr::null();
    }

    let image_data = unsafe { slice::from_raw_parts(buffer, length as usize) };
    let cursor = Cursor::new(image_data);

    let img = match ImageReader::new(cursor).with_guessed_format() {
        Ok(v) => match v.decode() {
            Ok(img) => img,
            Err(_) => return ptr::null(),
        },
        Err(_) => return ptr::null(),
    };

    println!("Imagem processada com sucesso");

    let xs = vec![img];

    let result = match model.run(&xs) {
        Ok((_, files)) => {
            if files.len() <= 0 {
                return ptr::null();
            }
            match CString::new(files[0].clone()) {
                Ok(v) => v.into_raw() as *const c_char,
                Err(_) => ptr::null(),
            }
        }
        Err(_) => ptr::null(),
    };

    result
}

#[no_mangle]
pub extern "C" fn free_c_string(s: *const c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s as *mut c_char);
    }
}
