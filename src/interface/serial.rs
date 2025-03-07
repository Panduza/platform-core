pub mod common;
pub mod eol;
pub mod settings;
pub mod slip;
pub mod time_lock;

pub use settings::Settings as SerialSettings;

pub use eol::SerialEolInterface;
