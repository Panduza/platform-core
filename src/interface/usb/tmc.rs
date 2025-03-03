use super::Settings as UsbSettings;
// use crate::protocol::BinaryCmdRespProtocol;
// use crate::std::class::repl::ReplProtocol;
use crate::{
    format_driver_error, log_trace, log_warn, protocol::BytesDialogProtocol, Error, Logger,
};
use async_trait::async_trait;
use byteorder::{ByteOrder, LittleEndian};

use bytes::Bytes;
use nusb::Interface as UsbInterface;
use std::sync::Arc;
use tokio::sync::Mutex;

///
///
pub struct UsbTmcInterface {
    ///
    /// To help data logging inside the driver
    ///
    logger: Logger,

    usb_interface: UsbInterface,

    endpoint_in: u8,
    endpoint_out: u8,
    max_packet_size_in: usize,

    /// Index of the next out request
    b_tag_index: u8,

    ///
    out_buffers: Vec<(usize, Vec<u8>)>,
    out_buffers_count: usize,
}

impl UsbTmcInterface {
    ///
    ///
    pub fn into_arc_mutex(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }

    /// Create a new instance of the driver
    ///
    pub fn open(settings: &UsbSettings) -> Result<Self, Error> {
        //
        // Prepare logger
        let logger = Logger::new_for_driver("usb", "tmc");

        // Find the USB device
        let dev = settings.find_usb_device().ok_or(Error::DriverError(
            "Unable to find the USB device".to_string(),
        ))?;

        let device: nusb::Device = match dev.open() {
            Ok(val) => val,
            Err(_e) => return Err(format_driver_error!("Unable to open USB device")),
        };

        let interface: Option<UsbInterface> = match device.claim_interface(0) {
            Ok(val) => Some(val),
            Err(_e) => {
                return Err(format_driver_error!(
                    "Unable to create USB device interface"
                ))
            }
        };

        // Find the IN endpoint in the configuration
        let (endpoint_in, max_packet_size_in) =
            Self::find_endpoint_in_config(&logger, interface.as_ref().unwrap()).unwrap();

        // Find the OUT endpoint in the configuration
        let (endpoint_out, max_packet_size_out) =
            Self::find_endpoint_out_config(&logger, interface.as_ref().unwrap()).unwrap();

        // let max_packet_size = endpoint_descriptor.max_packet_size() as usize;

        Ok(Self {
            logger: logger,
            usb_interface: interface.unwrap(),
            endpoint_in: endpoint_in,
            endpoint_out: endpoint_out,
            max_packet_size_in: max_packet_size_in,
            b_tag_index: 0,
            out_buffers: vec![
                (0, vec![0; max_packet_size_out]),
                (0, vec![0; max_packet_size_out]),
                (0, vec![0; max_packet_size_out]),
            ],
            out_buffers_count: 0,
        })
    }

    /// Find the in endpoint IN the configuration
    ///
    fn find_endpoint_in_config(
        logger: &Logger,
        interface: &nusb::Interface,
    ) -> Result<(u8, usize), Error> {
        for desc in interface.descriptors() {
            for endpoint in desc.endpoints() {
                if endpoint.direction() == nusb::transfer::Direction::In
                    && endpoint.transfer_type() == nusb::transfer::EndpointType::Bulk
                {
                    // If the endpoint is not 0x81, log a warning
                    // and continue, it can be a problem
                    if endpoint.address() != 0x81 {
                        log_warn!(
                            logger,
                            "Endpoint address is not 0x81, but {}",
                            endpoint.address()
                        );
                    }

                    // Trace the endpoint found and return configuration
                    log_trace!(logger, "In Endpoint found: {:?}", endpoint);
                    return Ok((endpoint.address(), endpoint.max_packet_size() as usize));
                }
            }
        }

        // If no endpoint is found, return an error
        Err(format_driver_error!(
            "Unable to find the IN endpoint in the USB device configuration"
        ))
    }

    /// Find the in endpoint OUT the configuration
    ///
    fn find_endpoint_out_config(
        logger: &Logger,
        interface: &nusb::Interface,
    ) -> Result<(u8, usize), Error> {
        for desc in interface.descriptors() {
            for endpoint in desc.endpoints() {
                if endpoint.direction() == nusb::transfer::Direction::Out
                    && endpoint.transfer_type() == nusb::transfer::EndpointType::Bulk
                {
                    // If the endpoint is not 0x02, log a warning
                    // and continue, it can be a problem
                    if endpoint.address() != 0x02 {
                        log_warn!(
                            logger,
                            "Endpoint address is not 0x02, but {}",
                            endpoint.address()
                        );
                    }

                    // Trace the endpoint found and return configuration
                    log_trace!(logger, "Out Endpoint found: {:?}", endpoint);
                    return Ok((endpoint.address(), endpoint.max_packet_size() as usize));
                }
            }
        }

        // If no endpoint is found, return an error
        Err(format_driver_error!(
            "Unable to find the OUT endpoint in the USB device configuration"
        ))
    }

    /// Increment b_tag and return the new value
    ///
    fn next_b_tag(&mut self) -> u8 {
        self.b_tag_index = (self.b_tag_index % 255) + 1;
        self.b_tag_index
    }

    /// Prepare the sequence of bulk_out requests
    ///
    fn prepare_request_sequence(&mut self, data: &[u8]) {
        //
        let b_tag = self.next_b_tag();
        Self::prepare_first_bulk_out_request_message(&mut self.out_buffers[0], b_tag, data);

        Self::prepare_bulk_in_request_message(&mut self.out_buffers[1], b_tag);

        self.out_buffers_count = 2;
    }

    /// Prepare the first bulk_out message
    ///
    fn prepare_first_bulk_out_request_message(
        out_buffer: &mut (usize, Vec<u8>),
        b_tag: u8,
        data: &[u8],
    ) {
        //
        // Prepare logger
        let logger = Logger::new_isolated("prepare_first_bulk_out_request_message");
        log_trace!(
            logger,
            "Prepare first bulk out (max size ={:?} Bytes) for (len={:?} Bytes) message={:?}",
            out_buffer.1.len(),
            data.len(),
            data
        );

        //
        // Initialize attributes
        let mut bm_transfer_attributes = 0x00;

        //
        // out_buffer big enough
        if out_buffer.1.len() >= (data.len() + 12) {
            bm_transfer_attributes = 0x01; // EOM (end of message)
            out_buffer.1[12..12 + data.len()].copy_from_slice(data);

            out_buffer.0 = 12 + data.len();

            // Manage padding on 32bits
            let need_padding = 4 - (out_buffer.0 % 4);
            if need_padding > 0 {
                for i in 0..need_padding {
                    out_buffer.1[out_buffer.0 + i] = 0;
                }
                out_buffer.0 += need_padding;
            }
        }
        // out_buffer not big enough
        else {
            let last_index = out_buffer.1.len() - 12;
            out_buffer.1[12..&data[..last_index].len() + 12].copy_from_slice(&data[..last_index]);
            out_buffer.0 = out_buffer.1.len();
        }

        // TMC Header
        out_buffer.1[0] = 1; // DevDepMsgOut
        out_buffer.1[1] = b_tag;
        out_buffer.1[2] = !b_tag & 0xFF; // b_tag_inverse
        out_buffer.1[3] = 0x00;

        // Out Header
        let transfer_size: usize = data.len();
        LittleEndian::write_u32(&mut out_buffer.1[4..8], transfer_size as u32);
        out_buffer.1[8] = bm_transfer_attributes;
        out_buffer.1[9] = 0x00;
        out_buffer.1[10] = 0x00;
        out_buffer.1[11] = 0x00;
    }

    ///
    ///
    fn prepare_bulk_in_request_message(out_buffer: &mut (usize, Vec<u8>), b_tag: u8) {
        // TMC Header
        out_buffer.1[0] = 2; // DevDepMsgIn
        out_buffer.1[1] = b_tag;
        out_buffer.1[2] = !b_tag & 0xFF; // b_tag_inverse
        out_buffer.1[3] = 0x00;

        // Out Header
        let transfer_size: usize = 1024 * 50;
        LittleEndian::write_u32(&mut out_buffer.1[4..8], transfer_size as u32);
        out_buffer.1[8] = 0x00;
        out_buffer.1[9] = 0x00;
        out_buffer.1[10] = 0x00;
        out_buffer.1[11] = 0x00;

        out_buffer.0 = 12;
    }

    ///
    ///
    fn parse_bulk_in_header(&self, data: &Vec<u8>) -> Result<usize, Error> {
        // log
        log_trace!(self.logger, "msg id: {}", data[0]);

        let transfer_size = LittleEndian::read_u32(&data[4..8]) as usize;

        Ok(transfer_size)
    }

    /// Perform echanges with the device
    ///
    pub async fn send_command(&mut self, command: &[u8]) -> Result<(), Error> {
        //
        // Prepare Request Sequence
        self.prepare_request_sequence(command);

        //
        // Send the sequence on the usb
        for i in 0..self.out_buffers_count {
            // Prepare data to be send
            let packet_size = self.out_buffers[i].0;
            let message = self.out_buffers[i].1[..packet_size].to_vec();
            log_trace!(
                self.logger,
                "BULK_OUT {:?} ({:?} Bytes) > {:?}",
                i,
                packet_size,
                &message
            );

            // SEND TO USB
            match self
                .usb_interface
                .bulk_out(self.endpoint_out, message)
                .await
                .into_result()
            {
                Ok(val) => {
                    log_trace!(self.logger, "BULK_OUT response {:?}", &val);
                }
                Err(_e) => return Err(format_driver_error!("Unable to write on USB")),
            };
        }

        Ok(())
    }

    /// Perform echanges with the device
    ///
    pub async fn execute_command(
        &mut self,
        command: &[u8],
        response: &mut Vec<u8>,
    ) -> Result<(), Error> {
        //
        // Prepare Request Sequence
        self.prepare_request_sequence(command);

        //
        // Send the sequence on the usb
        for i in 0..self.out_buffers_count {
            // Prepare data to be send
            let packet_size = self.out_buffers[i].0;
            let message = self.out_buffers[i].1[..packet_size].to_vec();
            log_trace!(
                self.logger,
                "BULK_OUT {:?} ({:?} Bytes) > {:?}",
                i,
                packet_size,
                &message
            );

            // SEND TO USB
            match self
                .usb_interface
                .bulk_out(self.endpoint_out, message)
                .await
                .into_result()
            {
                Ok(val) => {
                    log_trace!(self.logger, "BULK_OUT response {:?}", &val);
                }
                Err(_e) => return Err(format_driver_error!("Unable to write on USB")),
            };
        }

        //
        // Prepare bulk_in reception
        let mut is_first: bool = true;
        let mut remaining_data = 0;
        let mut is_eom = false;
        while !is_eom {
            // Prepare a new buffer
            let response_buffer = nusb::transfer::RequestBuffer::new(self.max_packet_size_in);

            // log
            log_trace!(self.logger, "BULK_IN wait...");

            // Receive data from the usb
            match tokio::time::timeout(
                std::time::Duration::from_secs(1),
                self.usb_interface
                    .bulk_in(self.endpoint_in, response_buffer),
            )
            .await
            {
                Ok(val) => match val.into_result() {
                    Ok(data) => {
                        //
                        //
                        if is_first {
                            // Parse the received data
                            let transfer_size = self.parse_bulk_in_header(&data).unwrap();

                            //
                            log_trace!(
                                self.logger,
                                "FIRST packet received transfert_total = {:?}",
                                transfer_size
                            );

                            is_first = false;
                            remaining_data = transfer_size + 12; // 12 => bulkin usbtmc header size

                            //
                            log_trace!(
                                self.logger,
                                "Remains = {:?} vs Data.len = {:?}",
                                remaining_data,
                                data.len()
                            );

                            if remaining_data >= data.len() {
                                remaining_data -= data.len();
                                response.extend(&data[12..]);
                            } else {
                                response.extend(&data[12..remaining_data]);
                                remaining_data = 0;
                            }
                        }
                        //
                        //
                        else {
                            // log
                            log_trace!(
                                self.logger,
                                "Data received (len:{:?}): {:?}",
                                data.len(),
                                data
                            );
                            log_trace!(self.logger, "Remaining data {:?}", remaining_data);

                            if remaining_data >= data.len() {
                                remaining_data -= data.len();

                                // Append the payload to the complete data
                                response.extend(data);
                            } else {
                                response.extend(&data[..remaining_data]);
                                remaining_data = 0;
                            }
                        }

                        // Check if this is the end of the message
                        if remaining_data > 0 {
                            is_eom = false;
                        } else {
                            is_eom = true;
                        }
                    }
                    Err(_e) => return Err(format_driver_error!("Unable to read on USB")),
                },
                Err(_) => {
                    return Err(format_driver_error!("Timeout while reading from USB"));
                }
            };
        }

        Ok(())
    }
}

#[async_trait]
///
///
impl BytesDialogProtocol for UsbTmcInterface {
    ///
    /// Just send a command and does not expect any response
    ///
    async fn tell(&mut self, command: Bytes) -> Result<(), Error> {
        self.send_command(&command).await
    }

    ///
    /// Send a command, wait for response and return it
    ///
    async fn ask(&mut self, command: Bytes) -> Result<Bytes, Error> {
        let mut response: Vec<u8> = Vec::new();
        self.execute_command(&command, &mut response).await?;
        Ok(Bytes::from(response))
    }
}

// #[async_trait]
// impl ReplProtocol for Driver {
//     /// Send a command and return the response as a string
//     ///
//     async fn eval(&mut self, command: String) -> Result<String, Error> {
//         // Log
//         log_trace!(self.logger, "Eval: {:?}", command);

//         // Execute command
//         let mut response = Vec::new();
//         self.execute_command(command.as_bytes(), &mut response)
//             .await?;

//         // Prepare
//         match String::from_utf8(response) {
//             Ok(s) => Ok(s),
//             Err(_) => Ok("Cannot convert the payload into string".to_string()),
//         }
//     }
// }
