use super::Settings as SerialSettings;
use crate::format_driver_error;
use crate::interface::serial::common;
use crate::log_debug;
use crate::log_trace;
use crate::Error;
use crate::Logger;
use serial2_tokio::SerialPort;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;

/// # Serial SLIP Driver
///
/// The goal of this driver is to manage the stack SLIP over SERIAL
///
/// ## What is SLIP ?
///
/// - [Wikipedia](https://en.wikipedia.org/wiki/Serial_Line_Internet_Protocol)
///
/// The Serial Line Internet Protocol (SLIP) is an encapsulation of the
/// Internet Protocol designed to work over serial ports and router connections.
/// It is documented in RFC 1055.
///
/// ## Why SLIP ?
///
/// This protocol helps splitting serial stream into packets.
/// You could just use EOL character driver but if you may have the
/// EOL char inside your payload data it becomes a problem.
/// SLIP works like EOL but provides a mecanism to avoid this problem
/// by encoding the payload with a simple et fast method.
///
pub struct Driver {
    ///
    ///
    ///
    pub logger: Logger,
    ///
    ///
    ///
    pub port: SerialPort,
    ///
    /// Read timeout
    ///
    read_timeout: Duration,
    ///
    /// Accumulated incoming data buffer
    ///
    in_buf: [u8; 2048],
    ///
    /// Keep track of number of data in the buffer
    ///
    in_buf_size: usize,
}

/// Connector is just a mutex protected driver
///
pub type Connector = Arc<Mutex<Driver>>;

impl Driver {
    /// Create a new instance of the driver
    ///
    pub fn open(settings: &SerialSettings) -> Result<Self, Error> {
        //
        // Open the port
        let (logger, port) = common::open(settings)?;
        //
        //
        log_debug!(logger, "SLIP !");
        //
        // Create instance
        Ok(Driver {
            logger: logger,
            port: port,
            read_timeout: settings.read_timeout,
            in_buf: [0u8; 2048],
            in_buf_size: 0,
        })
    }

    /// Lock the connector to write a command then wait for the answers
    ///
    pub async fn write_then_read(
        &mut self,
        command: &[u8],
        response: &mut [u8],
    ) -> Result<usize, Error> {
        Ok(
            timeout(self.read_timeout, self.__write_then_read(command, response))
                .await
                .map_err(|e| format_driver_error!("Timeout reading {:?}", e))??,
        )
    }

    /// This operation is not provided to the public interface
    /// User must use the timeout version for safety on the platform
    ///
    async fn __write_then_read(
        &mut self,
        command: &[u8],
        response: &mut [u8],
    ) -> Result<usize, Error> {
        // Prepare SLIP encoding
        // Prepare a buffer of 1024 Bytes (need to be change later TODO)
        // and prepare the encoder object
        let mut encoded_command = [0u8; 1024];
        let mut slip_encoder = serial_line_ip::Encoder::new();

        //
        // TRACE
        log_trace!(self.logger, "command before encoding - {:?}", command);

        // Encode the command
        let mut totals = slip_encoder
            .encode(command, &mut encoded_command)
            .map_err(|e| format_driver_error!("Unable to encode command: {:?}", e))?;

        // Finalise the encoding
        totals += slip_encoder
            .finish(&mut encoded_command[totals.written..])
            .map_err(|e| format_driver_error!("Unable to finsh command encoding: {:?}", e))?;

        let encoded_command_slice = &encoded_command[..totals.written];

        //
        // TRACE
        log_trace!(
            self.logger,
            "command after encoding - {:?}",
            encoded_command_slice
        );

        // Send the command
        let _write_result = self
            .port
            .write(encoded_command_slice)
            .await
            .map_err(|e| format_driver_error!("Unable to write on serial stream: {}", e))?;

        // Read the response until "end"
        loop {
            // Read a chunck
            self.in_buf_size += self
                .port
                .read(&mut self.in_buf[self.in_buf_size..])
                .await
                .map_err(|e| format_driver_error!("Unable to read on serial stream {:?}", e))?;

            //
            // TRACE
            log_trace!(
                self.logger,
                "response before decoding (size={:?}) - {:?}",
                self.in_buf_size,
                &self.in_buf[..self.in_buf_size]
            );

            // Try decoding
            let mut slip_decoder = serial_line_ip::Decoder::new();
            let (total_decoded, out_slice, end) = slip_decoder
                .decode(&self.in_buf[..self.in_buf_size], response)
                .map_err(|e| format_driver_error!("Unable to decode response: {:?}", e))?;

            //
            // TRACE
            log_trace!(self.logger, "end of packet - {:?}", end);

            // If a valid packet has been found, then we must return the out_slice len
            //      which is the len a the decoded data
            // Not '_total_decoded'
            //      because it is the number of byte processed from the encoded buffer
            if end {
                // Reset counter
                self.in_buf_size -= total_decoded;

                return Ok(out_slice.len());
            }
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn test_slip_decode() {
        const SLIP_ENCODED: [u8; 8] = [0xc0, 0x01, 0x02, 0x03, 0x04, 0x05, 0xc0, 0x04];
        const DATA: [u8; 5] = [0x01, 0x02, 0x03, 0x04, 0x05];

        let mut output: [u8; 32] = [0; 32];
        let mut slip = serial_line_ip::Decoder::new();

        let (input_bytes_processed, output_slice, is_end_of_packet) =
            slip.decode(&SLIP_ENCODED, &mut output).unwrap();

        assert_eq!(7, input_bytes_processed);
        assert_eq!(&DATA, output_slice);
        assert_eq!(true, is_end_of_packet);
    }
}
