#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CString;
use std::os::raw;

type EmacsFn = unsafe extern "C" fn(
    env: *mut emacs_env,
    nargs: isize,
    args: *mut emacs_value,
    arg1: *mut ::std::os::raw::c_void,
) -> emacs_value;

#[no_mangle]
#[allow(non_upper_case_globals)]
pub static plugin_is_GPL_compatible: libc::c_int = 0;

#[no_mangle]
pub extern "C" fn get_environment(ert: *mut emacs_runtime) -> *mut emacs_env {
    unsafe {
        let get_env = (*ert)
            .get_environment
            .expect("cannot get emacs enviornment");
        get_env(ert)
    }
}

#[no_mangle]
pub unsafe extern "C" fn emacs_module_init(ert: *mut emacs_runtime) -> libc::c_int {
    let env = get_environment(ert);

    println!("making function");
    let make_function = (*env).make_function.expect("cannot get make_function");
    let my_message = make_function(
        env,
        0,
        0,
        Some(message_from_rust),
        CString::new("This will print a nice message from Rust!")
            .unwrap()
            .as_ptr(),
        std::ptr::null_mut(),
    );

    let my_sum = make_function(
        env,
        0,
        10,
        Some(my_sum),
        CString::new("This will add all the numbers you pass to it!")
            .unwrap()
            .as_ptr(),
        std::ptr::null_mut(),
    );

    let intern = (*env).intern.expect("could not get intern in main");

    println!("interning my-rust-func");
    let fn_name = intern(env, CString::new("my-message-from-rust").unwrap().as_ptr());
    println!("interning my-rust-mod");
    let mod_name = intern(env, CString::new("my-rust-mod").unwrap().as_ptr());

    println!("interning sum_name");
    let sum_name = intern(env, CString::new("my-sum").unwrap().as_ptr());

    let fset = intern(env, CString::new("fset").unwrap().as_ptr());
    let provide = intern(env, CString::new("provide").unwrap().as_ptr());
    let fset_args = [fn_name, my_message].as_mut_ptr();

    let fset_second_args = [sum_name, my_sum].as_mut_ptr();

    let provide_args = [mod_name].as_mut_ptr();

    let funcall = (*env).funcall.expect("cannot get funcall");
    println!("calling fset");
    funcall(env, fset, 2, fset_args);
    println!("calling fset again");
    funcall(env, fset, 2, fset_second_args);
    println!("calling provide");
    funcall(env, provide, 1, provide_args);

    println!("end");

    0
}

unsafe fn provide(env: *mut emacs_env, feature: &str) {
    let feat = intern_sym(env, feature);
    let args = [feat].as_mut_ptr();
    funcall(env, "provide", 1, args);
}

#[no_mangle]
unsafe extern "C" fn message_from_rust(
    env: *mut emacs_env,
    nargs: isize,
    args: *mut emacs_value,
    data: *mut raw::c_void,
) -> emacs_value {
    let s = "Hello Emacs! I'm from Rust!";
    let make_string = (*env).make_string.unwrap();
    let c_string = CString::new(s).unwrap();
    let len = c_string.as_bytes().len() as isize;
    make_string(env, c_string.as_ptr(), len)
}

#[no_mangle]
unsafe extern "C" fn my_sum(
    env: *mut emacs_env,
    nargs: isize,
    args: *mut emacs_value,
    data: *mut raw::c_void,
) -> emacs_value {
    let extract_integer = (*env)
        .extract_integer
        .expect("could not get extract_integer");

    let mut ints = Vec::new();

    for i in 0..nargs {
        ints.push(extract_integer(env, *args.offset(i)));
    }

    let sum = ints.iter().fold(0, |acc, n| acc + n);

    let make_integer = (*env).make_integer.expect("could not get make_integer");

    make_integer(env, sum)
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

pub unsafe extern "C" fn intern_sym(env: *mut emacs_env, name: &str) -> emacs_value {
    (*env).intern.expect("cannot get intern")(
        env,
        CString::new(name).expect("cannot intern symbol").as_ptr(),
    )
}

pub unsafe extern "C" fn funcall(
    env: *mut emacs_env,
    name: &str,
    nargs: isize,
    args: *mut emacs_value,
) -> emacs_value {
    let qf = intern_sym(env, name);
    let funcall = (*env).funcall.expect("cannot get funcall");
    funcall(env, qf, nargs, args)
}

fn make_str(s: &str) -> *const i8 {
    CString::new(s).unwrap().as_ptr()
}
