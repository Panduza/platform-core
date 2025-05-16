//////////////////
///// TO DO //////
//////////////////
// - gérer les erreurs
// -savoir comment on veut set l'IP et le port de notre périphérique
//////////////////

// ethernet_settings.rs manages the settings for the ethernet interface such as IP and Port

//using serde_json to load ethernet settings from settings 
use serde_json::json;

/// Key for the IP of the device inside json settings
///
#[macro_export]
macro_rules! SETTINGS_ETHERNET_IP_KEY {
    () => {
        "IP"
    };
}

/// Key for the Port of the device inside json settings
///
#[macro_export]
macro_rules! SETTINGS_ETHERNET_PORT_KEY {
    () => {
        "Port"
    };
}

// ethernet settings for devices
pub struct Settings {
    /// IP address of the ethernet interface
    pub ip: Option<String>,
    /// Port of the ethernet interface
    /// u16 because port can be up to 65536
    pub port: Option<u16>,
}

impl Settings {
    /// Creates a new Settings instance
    ///
    pub fn new() -> Settings {
        Settings {
            ip: None,
            port: None,
        }
    }

    /// Set the IP address
    ///
    pub fn set_IP(mut self, ip:String) -> Self {
        self.ip = Some(ip);
        self
    }

    /// Set the Port
    ///
    pub fn set_Port(mut self, port:u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Look into a json settings object and try to extract ethernet configuration
    ///
    pub fn set_ethernet_settings_from_json_settings (
        mut self,
         settings :&serde_json::Value,
    ) -> Result<Self, Error>{
        
        // si présent dans la config alors on lui associe la valeurs
        // sinon on fait remonter l'erreur
        //IP
        self.ip = settings
            .get(SETTINGS_ETHERNET_IP_KEY)
            .ok_or(Error::BadSettings("Missing field 'ip'".into()))?
            .as_str()
            .ok_or(Error::BadSettings("'ip' must be a string".into()))?
            .to_string();

        //Port
        self.port = settings
            .get("port")
            .ok_or(Error::BadSettings("Missing field 'port'".into()))?
            .as_u64()
            .ok_or(Error::BadSettings("'port' must be a number".into()))?
            .as_u16();
        ok(self)
    }

}