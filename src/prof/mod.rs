use crate::ffi;
use crate::{Error, Result};

use std::ffi::CString;

/// A vAccel profile region
///
/// This describes a region in a vAccel application
/// for which we can gather stats
#[derive(Debug)]
pub struct ProfRegion {
    inner: ffi::vaccel_prof_region,
}

impl ProfRegion {
    /// Create a new profile region
    ///
    /// This will allocate and initialize a profile region
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the region
    pub fn new(name: &str) -> Result<Self> {
        let c_name = CString::new(name).map_err(|_| Error::InvalidArgument)?;

        let mut inner = ffi::vaccel_prof_region::default();

        match unsafe { ffi::vaccel_prof_region_init(&mut inner, c_name.as_c_str().as_ptr()) as u32 }
        {
            ffi::VACCEL_OK => Ok(ProfRegion { inner }),
            err => Err(Error::Runtime(err)),
        }
    }

    pub fn enter(&mut self) -> Result<()> {
        match unsafe { ffi::vaccel_prof_region_start(&mut self.inner) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    pub fn exit(&mut self) -> Result<()> {
        match unsafe { ffi::vaccel_prof_region_stop(&mut self.inner) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    pub fn print(&self) -> Result<()> {
        match unsafe { ffi::vaccel_prof_region_print(&self.inner) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }
}

impl Drop for ProfRegion {
    fn drop(&mut self) {
        unsafe { ffi::vaccel_prof_region_destroy(&mut self.inner) };
    }
}
