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

pub use panduza::fbs::notification_v0::NotificationBuffer;
pub use panduza::fbs::notification_v0::NotificationType;

/// Client public export
///
pub use panduza::fbs::number::NumberBuffer;
pub use panduza::fbs::status_v0::InstanceStatusBuffer;
pub use panduza::fbs::status_v0::StatusBuffer;
pub use panduza::TaskMonitor;

/// The engine is the core object that will handle the connections and the events
///
mod engine;
pub use engine::new_engine;
pub use engine::options::EngineOptions;
pub use engine::Engine;
pub use engine::EngineBuilder;

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
pub mod runtime;
pub use runtime::notification::attribute::AttributeMode;
pub use runtime::notification::group::NotificationGroup;
pub use runtime::notification::AlertNotification;
pub use runtime::notification::AttributeNotification;
pub use runtime::notification::ClassNotification;
pub use runtime::notification::Notification;
pub use runtime::notification::StateNotification;
pub use runtime::Runtime;
pub use runtime::RuntimeBuilder;

///
///
pub mod topic;
pub use topic::Topic;

///
///
pub mod env;

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
pub mod interface;

/// Currently we put here a trait waiting to see if there is a better use later
///
pub mod protocol;

///
///
pub mod helper;

///
///
pub mod model;

///
/// TODO => put in factory
///
pub mod props;
pub use props::Prop;
pub use props::PropType;
pub use props::Props;

///
///
pub mod template;
