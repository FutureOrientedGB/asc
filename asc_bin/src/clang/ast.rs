/* automatically generated by rust-bindgen 0.70.1 */

extern "C" {
    pub fn scan_necessary_sources(
        entry_point_file: *const ::std::os::raw::c_char,
        source_dir: *const ::std::os::raw::c_char,
        target_dir: *const ::std::os::raw::c_char,
        result_buf: *mut ::std::os::raw::c_char,
        result_len: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
