#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::*;

    #[test]
    fn it_works() {
        let ffi_cdda_version = unsafe { CStr::from_ptr(cdda_version()) };
        let ffi_paranoia_version = unsafe { CStr::from_ptr(paranoia_version()) };

        let cdda_version = ffi_cdda_version.to_str().expect("non-UTF8 version string");
        let paranoia_version = ffi_paranoia_version
            .to_str()
            .expect("non-UTF8 version string");

        eprintln!("cdda version: {}", cdda_version);
        eprintln!("paranoia version: {}", paranoia_version);
    }
}
