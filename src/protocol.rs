use crate::Error;
use async_trait::async_trait;

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
