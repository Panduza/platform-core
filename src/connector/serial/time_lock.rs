use super::{common, Settings as SerialSettings};
use crate::protocol::AsciiCmdRespProtocol;
use crate::{format_driver_error, log_debug, log_trace, DriverLogger, Error};
use async_trait::async_trait;
use serial2_tokio::SerialPort;
use std::time::Duration;
use tokio::io::AsyncReadExt;

/// TimeLock structure
///
pub struct TimeLock {
    pub duration: tokio::time::Duration,
    pub t0: tokio::time::Instant,
}

/// # Timelock Serial Driver
///
/// This driver must be used only for very broken devices that does not send EOF or \n
/// at the end of there message packets.
///
pub struct Driver {
    ///
    /// To help data logging inside the driver
    ///
    logger: DriverLogger,
    ///
    /// The serial port object
    ///
    port: SerialPort,
    ///
    /// A buffer to read incoming data
    ///
    read_buffer: [u8; 1024],
    ///
    /// The duration of the timelock
    ///
    time_lock_duration: Duration,
    ///
    /// If not none the current operation is time locked
    /// the driver must wait because newt operation
    ///
    time_lock: Option<TimeLock>,
}

impl Driver {
    /// Create a new instance of the driver
    ///
    pub fn open(settings: &SerialSettings, time_lock_duration: Duration) -> Result<Self, Error> {
        //
        // Open the port
        let (logger, port) = common::open(settings)?;
        //
        //
        log_debug!(logger, "Time Locked ! {:?}", time_lock_duration);
        //
        //
        Ok(Self {
            logger: logger,
            port: port,
            read_buffer: [0; 1024],
            time_lock_duration: time_lock_duration,
            time_lock: None,
        })
    }

    /// Write a command on the serial stream
    ///
    pub async fn write_time_locked(&mut self, command: &[u8]) -> Result<usize, Error> {
        // Check if a time lock is set
        if let Some(lock) = self.time_lock.as_mut() {
            let elapsed = tokio::time::Instant::now() - lock.t0;
            if elapsed < lock.duration {
                let wait_time = lock.duration - elapsed;
                tokio::time::sleep(wait_time).await;
            }
            self.time_lock = None;
        }

        // Send the command
        let write_result = self
            .port
            .write(command)
            .await
            .map_err(|e| format_driver_error!("Unable to write on serial port: {}", e));

        // Set the time lock
        self.time_lock = Some(TimeLock {
            duration: self.time_lock_duration,
            t0: tokio::time::Instant::now(),
        });

        return write_result;
    }

    ///
    ///
    ///
    async fn read_one_by_one(&mut self) -> Result<usize, Error> {
        let mut n = 0;
        loop {
            let mut single_buf = [0u8; 1];

            // timeout here with small time
            let operation_result = tokio::time::timeout(
                self.time_lock_duration,
                self.port.read_exact(&mut single_buf),
            )
            .await;

            match operation_result {
                Ok(read_result) => {
                    if let Err(e) = read_result {
                        return Err(format_driver_error!(
                            "Unable to read one more on serial port {:?}",
                            e
                        ));
                    }
                    self.read_buffer[n] = single_buf[0];
                    n += 1;
                }
                Err(_) => {
                    //
                    // Debug
                    log_trace!(self.logger, "Read {:?}", self.read_buffer[..n].to_vec());
                    return Ok(n);
                }
            }
        }
    }

    /// Lock the connector to write a command then wait for the answers
    ///
    pub async fn write_then_read_after(&mut self, command: &[u8]) -> Result<usize, Error> {
        // trace
        log_trace!(self.logger, "write {:?}", command);

        // Write
        self.write_time_locked(command).await?;

        // read
        self.read_one_by_one().await
    }
}

#[async_trait]
impl AsciiCmdRespProtocol for Driver {
    ///
    /// Send a command and go
    ///
    async fn send(&mut self, command: &String) -> Result<(), Error> {
        //
        // Append EOL to the command
        let command_buffer = command.clone().into_bytes();

        //
        // Write
        self.write_time_locked(command_buffer.as_slice()).await?;
        Ok(())
    }

    ///
    /// Send a command and expect a result
    ///
    async fn ask(&mut self, command: &String) -> Result<String, Error> {
        //
        // Append EOL to the command
        let command_buffer = command.clone().into_bytes();

        //
        // Read
        let count = self
            .write_then_read_after(command_buffer.as_slice())
            .await?;

        //
        // Build response string
        unsafe {
            let string_slice = String::from_utf8_unchecked(self.read_buffer[..count].to_vec());
            return Ok(string_slice.to_string());
        }
    }
}
