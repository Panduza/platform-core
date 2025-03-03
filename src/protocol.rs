use crate::Error;
use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
///
/// Protocol in which we send a text command and the device respond with another text
///
pub trait AsciiCmdRespProtocol: Sync + Send {
    ///
    /// Just send a command and does not expect any response
    ///
    async fn send(&mut self, command: &String) -> Result<(), Error>;

    ///
    /// Send a command and return the response
    ///
    async fn ask(&mut self, command: &String) -> Result<String, Error>;
}

#[async_trait]
///
/// Protocol in which we send a binary command and the device respond with another binary
///
pub trait BinaryCmdRespProtocol: Sync + Send {
    ///
    /// Just send a command and does not expect any response
    ///
    async fn send(&mut self, command: &[u8]) -> Result<(), Error>;

    ///
    /// Send a command and return the response
    ///
    async fn ask(&mut self, command: &[u8], response: &mut [u8]) -> Result<usize, Error>;
}

#[async_trait]
/// Synchrone Bytes Dialog Protocol
///
pub trait BytesDialogProtocol: Sync + Send {
    ///
    /// Just send a command and does not expect any response
    ///
    async fn tell(&mut self, command: Bytes) -> Result<(), Error>;

    ///
    /// Send a command, wait for response and return it
    ///
    async fn ask(&mut self, command: Bytes) -> Result<Bytes, Error>;
}
