/*
    Utility functions

    Warning: some of these functions are Linux-specific.
*/

pub mod either;
pub mod file_util;
pub mod network_util;
pub mod process_util;
pub mod time_util;

/*
    Very bad function -- leaks the string memory :)
    Only use for e.g. command line arguments where it won't happen repeatedly.
*/

pub fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}
