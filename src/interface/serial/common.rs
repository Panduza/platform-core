use super::Settings as SerialSettings;
use crate::format_driver_error;
use crate::log_debug;
use crate::{DriverLogger, Error};
use serial2_tokio::SerialPort;
use serial2_tokio::Settings;

/// Create a new instance of the driver
///
pub fn open(settings: &SerialSettings) -> Result<(DriverLogger, SerialPort), Error> {
    //
    // Get the port name safely
    let port_name = settings
        .port_name
        .as_ref()
        .map(|val| val.clone())
        .unwrap_or("undefined".to_string())
        .clone();

    //
    // Prepare logger
    let logger = DriverLogger::new("serial", "generic", &port_name);
    log_debug!(logger, "Opening serial driver {:?}...", &port_name);

    //
    // Open port
    let port = SerialPort::open(&port_name, |mut serial2_settings: Settings| {
        serial2_settings.set_baud_rate(settings.baudrate)?;
        serial2_settings.set_char_size(settings.data_bits);
        serial2_settings.set_stop_bits(settings.stop_bits);
        serial2_settings.set_parity(settings.parity);
        serial2_settings.set_flow_control(settings.flow_control);

        log_debug!(logger, "Settings used : {:?}", &serial2_settings);

        Ok(serial2_settings)
    })
    .map_err(|e| format_driver_error!("Port {:?} {:?}", &port_name, e))?;

    //
    // Debug logs
    log_debug!(logger, "Open success !");
    if let Ok(port_settings) = port.get_configuration() {
        if let Ok(baudrate) = port_settings.get_baud_rate() {
            log_debug!(logger, "- Baudrate {:?}...", baudrate);
        }
    }

    // return objects
    Ok((logger, port))
}
