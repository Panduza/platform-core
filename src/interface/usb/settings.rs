use crate::Error;
use nusb::DeviceInfo;
use serde_json::json;

/// Key for usb vid inside json settings
///
#[macro_export]
macro_rules! SETTINGS_USB_VID_KEY {
    () => {
        "usb_vid"
    };
}

/// Key for usb pid inside json settings
///
#[macro_export]
macro_rules! SETTINGS_USB_PID_KEY {
    () => {
        "usb_pid"
    };
}

/// Key for usb serial string inside json settings
///
#[macro_export]
macro_rules! SETTINGS_USB_SERIAL_KEY {
    () => {
        "usb_serial"
    };
}

/// Usb settings for devices
#[derive(Debug)]
pub struct Settings {
    ///
    /// VID
    ///
    pub vid: Option<u16>,

    ///
    /// PID
    ///
    pub pid: Option<u16>,

    ///
    /// Serial String
    ///
    pub serial: Option<String>,
}

impl Settings {
    /// Creates a new Settings instance
    ///
    pub fn new() -> Settings {
        Settings {
            vid: None,
            pid: None,
            serial: None,
        }
    }

    /// Set the vendor
    ///
    pub fn set_vendor(mut self, vendor: u16) -> Self {
        self.vid = Some(vendor);
        self
    }

    /// Set the model
    ///
    pub fn set_model(mut self, model: u16) -> Self {
        self.pid = Some(model);
        self
    }

    ///
    ///
    pub fn set_serial_from_json_settings(
        mut self,
        settings: &serde_json::Value,
    ) -> Result<Self, Error> {
        self.serial = Some(
            settings
                .get(SETTINGS_USB_SERIAL_KEY!())
                .ok_or(Error::BadSettings(format!(
                    "Unable to get \"{}\"",
                    SETTINGS_USB_SERIAL_KEY!()
                )))?
                .as_str()
                .ok_or(Error::BadSettings(format!(
                    "\"{}\" not a string",
                    SETTINGS_USB_SERIAL_KEY!()
                )))?
                .to_string(),
        );
        Ok(self)
    }

    /// Like `set_serial_from_json_settings` but with a default value in case
    /// of error on settings extraction
    ///
    pub fn set_serial_from_json_settings_or(
        mut self,
        settings: &serde_json::Value,
        default: &str,
    ) -> Self {
        let default_as_value = json!(default);
        self.serial = Some(
            settings
                .get(SETTINGS_USB_SERIAL_KEY!())
                .unwrap_or_else(|| &default_as_value)
                .as_str()
                .unwrap_or_else(|| default)
                .to_string(),
        );
        self
    }

    /// Look into a json settings object and try to extract usb configuration
    ///
    ///
    pub fn optional_set_serial_from_json_settings(mut self, settings: &serde_json::Value) -> Self {
        if let Some(vendor) = settings.get(SETTINGS_USB_VID_KEY!()) {
            if let Some(s) = vendor.as_u64() {
                self.vid = Some(s as u16);
            }
        }
        if let Some(model) = settings.get(SETTINGS_USB_PID_KEY!()) {
            if let Some(s) = model.as_u64() {
                self.pid = Some(s as u16);
            }
        }
        self.serial = match settings.get(SETTINGS_USB_SERIAL_KEY!()) {
            Some(serial) => match serial.as_str() {
                Some(s) => Some(s.to_string()),
                None => None,
            },
            None => None,
        };
        self
    }

    /// Look into a json settings object and try to extract usb configuration
    ///
    pub fn from_json_settings(json_settings: &serde_json::Value) -> Self {
        //
        // Try to extract vid
        let vid = json_settings
            .get(SETTINGS_USB_VID_KEY!())
            .and_then(|vendor| vendor.as_u64())
            .map(|s| s as u16);

        //
        // Try to extract pid
        let pid = json_settings
            .get(SETTINGS_USB_PID_KEY!())
            .and_then(|model| model.as_u64())
            .map(|s| s as u16);

        //
        // Try to extract serial number
        let serial = json_settings
            .get(SETTINGS_USB_PID_KEY!())
            .and_then(|serial| serial.as_str())
            .map(|s| s.to_string());

        //
        // Return Object
        Self { vid, pid, serial }
    }

    ///
    ///
    ///
    pub fn find_usb_device(&self) -> Option<DeviceInfo> {
        //
        //
        let mut found_device = None;
        //
        //
        for dev in nusb::list_devices().unwrap() {
            //
            //
            if let Some(v_vid) = self.vid {
                if dev.vendor_id() != v_vid {
                    continue;
                }
            }
            //
            //
            if let Some(v_pid) = self.pid {
                if dev.product_id() != v_pid {
                    continue;
                }
            }

            found_device = Some(dev);
        }
        found_device
    }
}

impl std::fmt::Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let vendor = self.vid.unwrap_or(0);
        let model = self.pid.unwrap_or(0);
        write!(
            f,
            "Settings {{ vendor: {:#02x}, model: {:#02x}, serial: {:?} }}",
            vendor, model, self.serial
        )
    }
}
