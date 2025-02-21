use crate::{log_error, log_info, tracing::Logger};

///
///
///
pub fn list_all(vid: Option<u16>, pid: Option<u16>) {
    //
    //
    let logger = Logger::new("SDK", "", "", "");

    //
    //
    for dev in nusb::list_devices().unwrap() {
        //
        //
        if let Some(v_vid) = vid {
            if dev.vendor_id() != v_vid {
                continue;
            }
        }
        //
        //
        if let Some(v_pid) = pid {
            if dev.product_id() != v_pid {
                continue;
            }
        }

        //
        //
        log_info!(logger, "{:#?}", dev);

        //
        //
        let dev = match dev.open() {
            Ok(dev) => dev,
            Err(e) => {
                log_error!(logger, "Failed to open device: {}", e);
                continue;
            }
        };

        match dev.active_configuration() {
            Ok(config) => log_info!(
                logger,
                "Active configuration is {}",
                config.configuration_value()
            ),
            Err(e) => log_error!(logger, "Unknown active configuration: {e}"),
        }

        for config in dev.configurations() {
            log_info!(logger, "{config:#?}");
        }
    }
}
