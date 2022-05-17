extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rustc-link-lib=apr-1");
    println!("cargo:rerun-if-changed=apache2_headers.h");
    let apache2_bindings = bindgen::Builder::default()
        .header("apache2_headers.h")
        .clang_arg("-I/usr/include/apr-1.0")
        .clang_arg("-I/usr/include/apache2")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings for apache2");
    apache2_bindings
        .write_to_file(out_path.join("apache2_bindings.rs"))
        .expect("Could not write bindings for apache2");

    println!("cargo:rerun-if-changed=render_protocol_headers.h");
    let renderd_protocol_bindings = bindgen::Builder::default()
        .header("renderd_protocol.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings for renderd protocol");
    renderd_protocol_bindings
        .write_to_file(out_path.join("renderd_protocol_bindings.rs"))
        .expect("Could not write bindings for renderd protocol")
}
