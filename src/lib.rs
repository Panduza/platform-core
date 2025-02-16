//! # Panduza Platform Core
//!
//! This crate is the heart of Panduza platform and plugins
//!

// #![deny(
//     while_true,
//     improper_ctypes,
//     non_shorthand_field_patterns,
//     no_mangle_generic_items,
//     overflowing_literals,
//     path_statements,
//     patterns_in_fns_without_body,
//     unconditional_recursion,
//     bad_style,
//     dead_code,
//     unused,
//     unused_allocation,
//     unused_comparisons,
//     unused_parens
// )]

/// Main error crate for Panduza Platform
///
mod error;
pub use error::Error;

/// Module that manage platform traces and logs
///
/// ## Debug level policy
///
/// Those logs will always be in logs for release build mode.
/// They have to provides as much information as possible to debug
/// without impacting system performances.
///
/// ## Trace level policy
///
/// Those logs are available only on debug build mode.
/// They must be used only in developpement steps and deep investigations.
///
pub mod tracing;
pub use tracing::Logger; // only this one must stay at the end (others deprecated)

/// Client public export
///
pub use panduza::TaskMonitor;
/// The engine is the core object that will handle the connections and the events
///
mod engine;
pub use engine::options::EngineOptions;
pub use engine::Engine;

/// Plugin object
///
pub mod plugin;
pub use plugin::Plugin;

///
///
mod factory;
pub use factory::producer::Producer;
pub use factory::production_order::InstanceSettings;
pub use factory::production_order::ProductionOrder;
pub use factory::store::Product;
pub use factory::store::Store;
pub use factory::Factory;
// pub use factory::ScanMachine;

/// Manage an instance of a driver
///
pub mod instance;
pub use instance::actions::Actions;
pub use instance::container::Container;
pub use instance::Instance;

///
///
// pub mod pmacro;
pub mod topic;
pub use topic::Topic;

// pub use instance::class::Class;
// pub use instance::class_builder::ClassBuilder;
// pub use instance::container::Container;
// pub use instance::monitor::InstanceMonitor;
// pub use instance::Instance;
// pub use instance::InstanceInner;

//
// pub use instance::attribute::builder::AttributeServerBuilder;
// pub use instance::attribute::server_boolean::BooleanAttServer;
// pub use instance::attribute::server_enum::EnumAttServer;
// pub use instance::attribute::server_json::JsonAttServer;
// pub use instance::attribute::server_mem_cmd::MemoryCommandAttServer;
// pub use instance::attribute::server_number::NumberAttServer;
// pub use instance::attribute::server_si::SiAttServer;
// pub use instance::attribute::server_string::StringAttServer;

// public traits
// mod traits;
// pub use traits::DriverOperations;
// pub use traits::MessageCodec;
// pub use traits::MessageHandler;
// pub use traits::Producer;
// pub use traits::Scanner;

//
// mod codec;
// pub use codec::boolean::BooleanCodec;
// pub use codec::eenum::EnumCodec;
// pub use codec::json::JsonCodec;
// pub use codec::memory_command::MemoryCommandCodec;
// pub use codec::memory_command::MemoryCommandMode;
// pub use codec::number::NumberCodec;
// pub use codec::number_list::NumberListCodec;
// pub use codec::raw::RawCodec;
// pub use codec::si::SiCodec;
// pub use codec::string::StringCodec;
// pub use codec::string_list::StringListCodec;

/// Return type for spawned task
pub type TaskResult = Result<(), Error>;

// pub mod runtime;
// pub use runtime::Runtime;

pub mod env;

// pub use runtime::notification::attribute::AttributeMode;
// pub use runtime::notification::group::NotificationGroup;
// pub use runtime::notification::AlertNotification;
// pub use runtime::notification::AttributeNotification;
// pub use runtime::notification::ClassNotification;
// pub use runtime::notification::Notification;
// pub use runtime::notification::StateNotification;

/// Built-in Protocols & Interfaces to help coding plugins
///
/// # Protocols & Interfaces
///
/// Protocols are the way to communicate with the drivers.
/// You can stack them to create a chain of protocols.
///
/// Interfaces are implementations of the protocols.
///
/// Stack terms are lower (to physical layers) and upper (to user layers).
///
/// # Enablement
///
/// Specific features need to be activated to enable drivers
///
/// - usb => for usb drivers (also enable usb)
/// - serial => for serial drivers (also enable usb)
///
// pub mod connector;

/// Currently we put here a trait waiting to see if there is a better use later
///
pub mod protocol;

///
/// TODO => put in factory
///
pub mod props;
pub use props::Prop;
pub use props::PropType;
pub use props::Props;

///
///
///
// pub mod std;
pub mod stable_number;
pub use stable_number::StableNumber;
