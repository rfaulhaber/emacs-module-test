#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CString;
use std::os::raw;

#[no_mangle]
#[allow(non_upper_case_globals)]
pub static plugin_is_GPL_compatible: libc::c_int = 0;

#[no_mangle]
pub extern "C" fn get_environment(ert: *mut emacs_runtime) -> *mut emacs_env {
    unsafe {
        let get_env = (*ert).get_environment.unwrap();
        get_env(ert)
    }
}

#[no_mangle]
pub extern "C" fn emacs_module_init(ert: *mut emacs_runtime) -> libc::c_int {
    let env = get_environment(ert);

    unsafe {
        let make_function = (*env).make_function.unwrap();
        let f = make_function(
            env,
            0,
            0,
            Some(my_func),
            CString::new("This is a test function written in Rust!")
                .unwrap()
                .as_ptr(),
            [].as_mut_ptr(),
        );

        let intern = (*env).intern.unwrap();
        let fset = intern(env, CString::new("fset").unwrap().as_ptr());
        let sym = intern(env, CString::new("my-rust-func").unwrap().as_ptr());

        let funcall = (*env).funcall.unwrap();
        // basically the equivalent of (fset 'my-rust-func my_func)
        funcall(env, fset, 2, [sym, f].as_mut_ptr());
    }
    provide(env, "my-rust-mod".into());

    0
}

#[no_mangle]
pub extern "C" fn find_function(env: *mut emacs_env, name: &str) -> emacs_value {
    unsafe {
        let intern = (*env).intern.unwrap();
        intern(env, CString::new(name).unwrap().as_ptr())
    }
}
#[no_mangle]
pub extern "C" fn provide(env: *mut emacs_env, feature: String) {
    let feat = unsafe {
        let intern = (*env).intern.unwrap();
        intern(env, CString::new(feature).unwrap().as_ptr())
    };
    let provide = find_function(env, "provide");
    let args = [feat].as_mut_ptr();
    unsafe {
        let funcall = (*env).funcall.unwrap();
        funcall(env, provide, 1, args)
    };
}

// a rust way of writing (get-buffer-create "*hello world*")
#[no_mangle]
extern "C" fn my_func(
    env: *mut emacs_env,
    nargs: libc::ptrdiff_t,
    args: *mut emacs_value,
    data: *mut raw::c_void,
) -> emacs_value {
    let get_buffer_create = find_function(env, "get-buffer-create");
    let args = [make_emacs_string(env, "*hello world*")].as_mut_ptr();

    unsafe {
        let funcall = (*env).funcall.unwrap();
        funcall(env, get_buffer_create, 1, args)
    }
}

pub extern "C" fn make_emacs_string<S>(env: *mut emacs_env, string: S) -> emacs_value
where
    S: Into<Vec<u8>>,
{
    let c_string = CString::new(string).unwrap().as_ptr();
    unsafe {
        let strlen = libc::strlen(c_string) as isize;
        let make_string = (*env).make_string.unwrap();
        make_string(env, c_string, strlen)
    }
}
