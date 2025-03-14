use super::{common, SerialSettings};
use crate::protocol::BytesDialogProtocol;
use crate::{format_driver_error, log_debug, log_trace, Error, Logger};
use async_trait::async_trait;
use bytes::Bytes;
use serial2_tokio::SerialPort;
use std::str;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;

///
///
pub struct SerialEolInterface {
    ///
    /// To help data logging inside the driver
    ///
    logger: Logger,
    ///
    /// The serial port object
    ///
    port: SerialPort,
    ///
    /// End of line
    ///
    eol: Vec<u8>,
    ///
    /// Read timeout
    ///
    read_timeout: Duration,
    ///
    ///
    ///
    read_buffer: [u8; 1024],
}

impl SerialEolInterface {
    /// Create a new instance of the driver
    ///
    pub fn open(settings: &SerialSettings, eol: Vec<u8>) -> Result<Self, Error> {
        //
        // Open the port
        let (logger, port) = common::open(settings)?;
        //
        //
        log_debug!(logger, "End Of Line ! {:?}", eol);
        //
        //
        Ok(Self {
            logger: logger,
            port: port,
            eol: eol,
            read_timeout: settings.read_timeout,
            read_buffer: [0; 1024],
        })
    }

    ///
    ///
    pub fn into_arc_mutex(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }

    ///
    /// Perform a read operation and protect the operation against timeouts
    ///
    pub async fn read_until_timeout(&mut self) -> Result<usize, Error> {
        let operation_result = timeout(self.read_timeout, self.read_until()).await;
        match operation_result {
            Ok(read_result) => {
                return read_result;
            }
            Err(e) => return Err(format_driver_error!("Read timeout: {:?}", e)),
        }
    }

    ///
    /// Perform a read operation and protect the operation against timeouts
    ///
    pub async fn read_until(&mut self) -> Result<usize, Error> {
        // Read the response until "end"
        let mut n = 0;
        loop {
            // let mut single_buf = [0u8; 1];
            let rx_count = self
                .port
                .read(&mut self.read_buffer[n..])
                .await
                .map_err(|e| format_driver_error!("Unable to read on serial port {:?}", e))?;
            // response[n] = single_buf[0];
            // n += rx_count;

            //
            // Debug
            // log_debug!(self.logger, "Read one {:?}", response[..n].to_vec());

            for _i in 0..rx_count {
                n += 1;
                if n >= self.eol.len() {
                    if self.read_buffer[n - self.eol.len()..n].to_vec() == *self.eol {
                        return Ok(n);
                    }
                }
            }
        }
    }
}

#[async_trait]
///
///
impl BytesDialogProtocol for SerialEolInterface {
    ///
    /// Just send a command and does not expect any response
    ///
    async fn tell(&mut self, command: Bytes) -> Result<(), Error> {
        //
        // For trace ONLY
        {
            let debug_conversion = str::from_utf8(&command);
            if let Ok(str_data) = debug_conversion {
                log_trace!(
                    self.logger,
                    "SerialEolInterface::tell({:?} - {:?})",
                    str_data,
                    &command.to_vec()
                );
            } else {
                log_trace!(
                    self.logger,
                    "SerialEolInterface::tell({:?})",
                    &command.to_vec()
                );
            }
        }

        //
        // Append EOL to the command
        let mut command_buffer = command.to_vec();
        command_buffer.extend(&self.eol);

        //
        // Write
        self.port
            .write(command_buffer.as_slice())
            .await
            .map_err(|e| format_driver_error!("Unable to write on serial port: {:?}", e))?;

        Ok(())
    }

    ///
    /// Send a command, wait for response and return it
    ///
    async fn ask(&mut self, command: Bytes) -> Result<Bytes, Error> {
        //
        // Append EOL to the command
        let mut command_buffer = command.to_vec();
        command_buffer.extend(&self.eol);

        //
        // TRACE
        {
            let debug_conversion = str::from_utf8(&command);
            if let Ok(str_data) = debug_conversion {
                log_trace!(
                    self.logger,
                    "SerialEolInterface::ask/query({:?} - {:?})",
                    str_data,
                    &command_buffer
                );
            } else {
                log_trace!(
                    self.logger,
                    "SerialEolInterface::ask/query({:?})",
                    &command_buffer
                );
            }
        }

        //
        // Write
        self.port
            .write(command_buffer.as_slice())
            .await
            .map_err(|e| format_driver_error!("Unable to write on serial port: {}", e))?;

        //
        // Read
        let count = self.read_until_timeout().await?;

        //
        // Build response string
        let response_slice = self.read_buffer[..count - self.eol.len()].to_vec();

        //
        // TRACE
        {
            let debug_conversion = str::from_utf8(&response_slice);
            if let Ok(str_data) = debug_conversion {
                log_trace!(
                    self.logger,
                    "SerialEolInterface::ask/answer({:?} - {:?})",
                    str_data,
                    &response_slice
                );
            }
        }

        Ok(Bytes::from(response_slice))
    }
}
