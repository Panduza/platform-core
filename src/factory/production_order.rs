use std::ffi::{c_char, CStr, CString};

use serde_json::json;
pub type InstanceSettings = serde_json::Value;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ProductionOrder {
    /// Name of the device to be produced
    pub name: String,

    /// Reference of driver device producer
    pub dref: String,

    ///
    pub settings: Option<InstanceSettings>,
}

impl ProductionOrder {
    /// Constructor
    ///
    pub fn new<A: Into<String>, B: Into<String>>(d_ref: A, d_name: B) -> ProductionOrder {
        ProductionOrder {
            name: d_name.into(),
            dref: d_ref.into(),
            settings: None,
        }
    }

    ///
    ///
    pub fn add_u16_setting<A: Into<String>>(mut self, name: A, setting: u16) -> Self {
        if self.settings.is_none() {
            self.settings = Some(json!({}));
        }

        let se = self.settings.as_mut().unwrap();
        let obj = se.as_object_mut().unwrap();
        obj.insert(name.into(), serde_json::Value::Number(setting.into()));

        self
    }

    ///
    ///
    pub fn add_string_setting<A: Into<String>, B: Into<String>>(
        mut self,
        name: A,
        setting: B,
    ) -> Self {
        if self.settings.is_none() {
            self.settings = Some(json!({}));
        }

        let se = self.settings.as_mut().unwrap();
        let obj = se.as_object_mut().unwrap();
        obj.insert(name.into(), serde_json::Value::String(setting.into()));

        self
    }

    /// From a json value
    ///
    // pub fn from_json(value: &serde_json::Value) -> ProductionOrder {
    //     ProductionOrder {
    //         name: "test".to_string(),
    //         dref: "rtok".to_string(),
    //         settings: None,
    //     }
    // }

    pub fn dref(&self) -> &String {
        &self.dref
    }

    /// Converts the ProductionOrder into a C string
    ///
    /// Don't forget "".as_c_str().as_ptr()" to use it with the DLL interfaces
    ///
    pub fn to_c_string(&self) -> Result<CString, crate::Error> {
        let json_str =
            serde_json::to_string(self).expect("Failed to serialize ProductionOrder to JSON");
        CString::new(json_str)
            .map_err(|e| crate::Error::InternalLogic(format!("Failed to build CString ({:?})", e)))
    }

    // /// Converts a C-style string pointer into a `ProductionOrder`
    pub fn from_c_str_ptr(c_str: *const c_char) -> Result<Self, crate::Error> {
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

        let json: serde_json::Value = serde_json::from_str(str)
            .map_err(|e| crate::Error::InvalidArgument(format!("Invalid JSON: {:?}", e)))?;

        let obj = serde_json::from_value(json).map_err(|e| {
            crate::Error::InvalidArgument(format!("Failed to deserialize JSON: {:?}", e))
        })?;

        Ok(obj)
    }
}
