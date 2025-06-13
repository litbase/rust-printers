use crate::common::base::printer::{PrintOptions, PrintOrientation};
use crate::unix::cups::dests::CupsOptionT;
use crate::{
    common::traits::platform::PlatformPrinterJobGetters,
    unix::utils::{date::time_t_to_system_time, strings::{c_char_to_string, str_to_cstring}},
};
use libc::{c_char, c_int, time_t};
use std::ffi::CStr;
use std::{slice, time::SystemTime};

#[link(name = "cups")]
unsafe extern "C" {

    unsafe fn cupsPrintFile(
        printer_name: *const c_char,
        filename: *const c_char,
        title: *const c_char,
        num_options: c_int,
        options: *const CupsOptionT,
    ) -> i32;

    unsafe fn cupsGetJobs(
        jobs: *mut *mut CupsJobsS,
        name: *const c_char,
        myjobs: c_int,
        whichjobs: c_int,
    ) -> c_int;

}

#[derive(Debug)]
#[repr(C)]
pub struct CupsJobsS {
    id: c_int,
    dest: *const c_char,
    title: *const c_char,
    user: *const c_char,
    format: *const c_char,
    state: c_int,
    size: c_int,
    priority: c_int,
    completed_time: time_t,
    creation_time: time_t,
    processing_time: time_t,
}

impl PlatformPrinterJobGetters for CupsJobsS {
    fn get_id(&self) -> u64 {
       return self.id as u64;
    }

    fn get_name(&self) -> String {
        return c_char_to_string(self.title);
    }

    fn get_state(&self) -> u64 {
        return self.state as u64;
    }

    fn get_printer(&self) -> String {
        return c_char_to_string(self.dest);
    }

    fn get_media_type(&self) -> String {
        return c_char_to_string(self.format);
    }

    fn get_created_at(&self) -> SystemTime {
        return time_t_to_system_time(self.creation_time).unwrap();
    }

    fn get_processed_at(&self) -> Option<SystemTime> {
        return time_t_to_system_time(self.processing_time);
    }

    fn get_completed_at(&self) -> Option<SystemTime> {
        return time_t_to_system_time(self.completed_time);
    }
}

/**
 * Return the printer jobs
 */
pub fn get_printer_jobs(printer_name: &str, active_only: bool) -> Option<&'static [CupsJobsS]> {
    let mut jobs_ptr: *mut CupsJobsS = std::ptr::null_mut();
    let whichjobs = if active_only { 0 } else { -1 };
    let name = str_to_cstring(printer_name);

    return unsafe {
        let jobs_count = cupsGetJobs(&mut jobs_ptr, name.as_ptr(), 0, whichjobs);
        if jobs_count > 0 {
            Some(slice::from_raw_parts(jobs_ptr, jobs_count as usize))
        } else {
            None
        }
    };
}

// Based on:
// https://github.com/apple/cups/blob/a8968fc4257322b1e4e191c4bccedea98d7b053e/cups/cups.h#L166
const CUPS_ORIENTATION: &CStr = c"orientation-requested";
const CUPS_ORIENTATION_PORTRAIT: &CStr = c"3";
const CUPS_ORIENTATION_LANDSCAPE: &CStr = c"4";

/**
 * Send an file to printer
 */
pub fn print_file(printer_name: &str, file_path: &str, job_name: Option<&str>, options: PrintOptions) -> Result<(), &'static str> {
    let mut options_vec = vec![];
    
    if let Some(orientation) = options.orientation {
        let value = if orientation == PrintOrientation::Landscape {
            CUPS_ORIENTATION_LANDSCAPE.as_ptr() as _
        } else {
            CUPS_ORIENTATION_PORTRAIT.as_ptr() as _
        };
        
        options_vec.push(CupsOptionT {
            name: CUPS_ORIENTATION.as_ptr() as _,
            value
        })
    }
    
    unsafe {        
        let printer = &str_to_cstring(printer_name);
        let filename = str_to_cstring(file_path);
        let title = str_to_cstring(job_name.unwrap_or(file_path));
 
        let result = cupsPrintFile(printer.as_ptr(), filename.as_ptr(), title.as_ptr(), options_vec.len() as _, options_vec.as_ptr());
        return if result == 0 {
            Err("cupsPrintFile failed")
        } else {
            Ok(())
        }
    }
}
