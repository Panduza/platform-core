pub mod macro_helper;
use std::ffi::{c_char, CStr};

use crate::Store;

///
/// !!!!!
/// Increment this number after a Plugin structure modification
/// !!!!!
///
static C_INTERFACE_VERSION: u32 = 0;

///
/// This structure provides the plugin interface
///
/// It means that all the plugins have to provide this structure
///
#[repr(C)]
pub struct Plugin {
    ///
    /// Version of this structure which is the interface
    /// between plugins and platform
    ///
    pub c_interface_version: u32,

    ///
    ///
    pub name: *const c_char,

    ///
    ///
    ///
    pub version: *const c_char,

    ///
    /// Must be called to join the plugin thread
    ///
    pub join: unsafe extern "C" fn(),

    ///
    /// Return the list of all references managed by this plugin
    ///
    /// The returned list must be a json array of string
    ///
    pub store: unsafe extern "C" fn() -> *const c_char,

    ///
    /// Return the list of all instances available on the server
    ///
    pub scan: unsafe extern "C" fn() -> *const c_char,

    ///
    /// Produce a device matching the given json string configuration
    ///
    pub produce: unsafe extern "C" fn(*const c_char) -> u32,

    ///
    /// Return the notifications
    ///
    pub pull_notifications: unsafe extern "C" fn() -> *const c_char,
}

impl Plugin {
    pub fn new(
        name: &'static CStr,
        version: &CStr,
        join: unsafe extern "C" fn(),
        store: unsafe extern "C" fn() -> *const c_char,
        scan: unsafe extern "C" fn() -> *const c_char,
        produce: unsafe extern "C" fn(*const c_char) -> u32,
        pull_notifications: unsafe extern "C" fn() -> *const c_char,
    ) -> Self {
        Plugin {
            c_interface_version: C_INTERFACE_VERSION,
            name: name.as_ptr(),
            version: version.as_ptr(),
            join: join,
            store: store,
            scan: scan,
            produce: produce,
            pull_notifications: pull_notifications,
        }
    }

    // /// Converts the ProductionOrder into a C string
    // ///
    // /// Don't forget "".as_c_str().as_ptr()" to use it with the DLL interfaces
    // ///
    // pub fn to_c_string(&self) -> Result<CString, crate::Error> {
    //     let json_str =
    //         serde_json::to_string(self).expect("Failed to serialize ProductionOrder to JSON");
    //     CString::new(json_str)
    //         .map_err(|e| crate::Error::InternalLogic(format!("Failed to build CString ({:?})", e)))
    // }

    ///
    /// Converts a C-style string pointer into a `ProductionOrder`
    ///
    pub unsafe fn store_as_obj(&self) -> Result<Store, crate::Error> {
        let c_str = (self.store)();

        //
        //
        if c_str.is_null() {
            return Err(crate::Error::InvalidArgument(
                "Null C string pointer".to_string(),
            ));
        }

        //
        //
        let c_str = unsafe { CStr::from_ptr(c_str) };
        let str = c_str
            .to_str()
            .map_err(|e| crate::Error::InvalidArgument(format!("Invalid C string: {:?}", e)))?;

        // println!("{:?}", str);

        let json: serde_json::Value = serde_json::from_str(str)
            .map_err(|e| crate::Error::InvalidArgument(format!("Invalid JSON: {:?}", e)))?;

        let obj = serde_json::from_value(json).map_err(|e| {
            crate::Error::InvalidArgument(format!("Failed to deserialize JSON: {:?}", e))
        })?;

        Ok(obj)
    }
}
