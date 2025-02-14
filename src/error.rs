use thiserror::Error as ThisError;

#[derive(ThisError, Debug, Clone)]
pub enum Error {
    #[error("Cannot publish message ({pyl_size:?} bytes) on topic {topic:?} because {cause:?}")]
    PublishError {
        topic: String,
        pyl_size: usize,
        cause: String,
    },
    #[error("Cannot subscribe to the message attribute topic")]
    MessageAttributeSubscribeError(String),
    #[error("Internal weak pointer cannot be upgraded")]
    InternalPointerUpgrade,
    #[error("Invalid argument given to the function")]
    InvalidArgument(String),
    #[error("Internal logic lead to this error")]
    InternalLogic(String),
    #[error("Error when trying to spawn a task")]
    Spawn(String),
    #[error("One of the provided settings is wrong")]
    BadSettings(String),
    #[error("Error during serialization")]
    SerializeFailure(String),
    #[error("Error during deserialization")]
    DeserializeError(String),
    #[error("Error related to plugin management")]
    PluginError(String),
    #[error("Error managing a cross task channel")]
    ChannelError(String),
    #[error("Error")]
    Generic(String),

    #[error("The value is not among the enum choices")]
    EnumOutOfChoices(String),
    #[error("The value is out of range")]
    SiOutOfRange(String),

    #[error("Driver operation failure")]
    DriverError(String),
    #[error("We just don't know what happened")]
    Wtf,
}

#[macro_export]
macro_rules! format_settings_error {
    ($($arg:tt)*) => {
        Error::BadSettings(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! format_driver_error {
    ($($arg:tt)*) => {
        Error::DriverError(format!($($arg)*))
    };
}
