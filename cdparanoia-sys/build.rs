use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rustc-link-lib=cdda_interface");
    println!("cargo:rustc-link-lib=cdda_paranoia");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_var("MAXTRK")
        .allowlist_var("GENERIC_SCSI")
        .allowlist_var("COOKED_IOCTL")
        .allowlist_var("TEST_INTERFACE")
        .allowlist_var("SGIO_SCSI")
        .allowlist_var("SGIO_SCSI_BUGGY1")
        .allowlist_var("strerror_tr")
        .allowlist_type("TOC")
        .allowlist_function("is_audio")
        .allowlist_var("CD_.*")
        .allowlist_var("CDDA_.*")
        .allowlist_var("TR_.*")
        .allowlist_var("PARANOIA_.*")
        .allowlist_type("cdda_.*")
        .allowlist_type("cdrom_.*")
        .allowlist_function("cdrom_*")
        .allowlist_function("cdda_.*")
        .allowlist_function("paranoia_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");
}
