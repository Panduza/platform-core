pub mod producer;
use panduza::pubsub::Operator;
pub use producer::Producer;

pub mod production_order;
pub mod store;
use store::{Product, Store};
use tokio::sync::mpsc::Sender;

use crate::{Engine, Instance, Logger, ProductionOrder};
use std::{collections::HashMap, ffi::CString};

/// Factory to create devices from a configuration json
///
pub struct Factory {
    /// Local logger
    logger: Logger,
    /// List of known producers
    producers: HashMap<String, Box<dyn Producer>>,
}

impl Factory {
    /// Create a new factory
    ///
    pub fn new() -> Self {
        // New object
        let obj = Factory {
            logger: Logger::new_for_factory(),
            producers: HashMap::new(),
        };
        // Info log
        obj.logger.info("# Device factory initialization");
        // Load builtin device producers
        return obj;
    }

    /// Add multiple producers
    ///
    pub fn add_producers(&mut self, producers: Vec<Box<dyn Producer>>) {
        for producer in producers {
            self.add_producer(producer);
        }
    }

    /// Add a single producer
    pub fn add_producer(&mut self, producer: Box<dyn Producer>) {
        // Info log
        self.logger.info(format!(
            "   - producer - {}.{}",
            producer.manufacturer(),
            producer.model()
        ));

        self.producers.insert(
            format!("{}.{}", producer.manufacturer(), producer.model()),
            producer,
        );
    }

    /// # Store
    ///
    /// Return the information about driver that can be produced by this factory
    ///
    /// ## Json Structure
    ///
    /// {
    ///     "dref" {
    ///         "description": "....",
    ///         "props": {}
    ///     }
    /// }
    ///
    pub fn store(&self) -> Store {
        //
        // Init the store with default empty value
        let mut store = Store::default();

        //
        // Go through all producers to build product list
        for (dref, producer) in &self.producers {
            store.products.insert(
                dref.clone(),
                Product {
                    description: producer.description(),
                    props: producer.props(),
                },
            );
        }

        //
        // Return the collected store
        store
    }

    ///
    /// Convert the store object into a c_string to send it through C interface
    ///
    pub fn store_as_c_string(&self) -> Result<CString, crate::Error> {
        let json_str =
            serde_json::to_string(&self.store()).expect("Failed to serialize store to JSON");
        CString::new(json_str)
            .map_err(|e| crate::Error::InternalLogic(format!("Failed to build CString ({:?})", e)))
    }

    /// production_order => json with ref, name, settings
    ///
    pub fn produce(
        &self,
        engine: Engine,
        // r_notifier: Option<Sender<Notification>>,
        production_order: ProductionOrder,
    ) -> Instance {
        let producer = self.producers.get(production_order.dref()).unwrap();
        let instance_actions = producer.produce().unwrap();

        Instance::new(
            engine.clone(),
            production_order.name,
            instance_actions,
            None, // settings
        )
    }
}

// ///
// ///
// pub struct ScanMachine {
//     /// Local logger
//     logger: Logger,
//     ///
//     scanners: Vec<Box<dyn Scanner>>,
// }
// impl ScanMachine {
//     /// Create a new factory
//     ///
//     pub fn new() -> Self {
//         // New object
//         let obj = Self {
//             logger: Logger::new_for_factory(),
//             scanners: Vec::new(),
//         };
//         // Info log
//         obj.logger.info("# Scan Machine initialization");
//         // Load builtin device producers
//         return obj;
//     }
//     pub fn add_scanners(&mut self, scanners: Vec<Box<dyn Scanner>>) {
//         for scanner in scanners {
//             self.add_scanner(scanner);
//         }
//     }

//     /// Add a single producer
//     pub fn add_scanner(&mut self, scanner: Box<dyn Scanner>) {
//         // Info log
//         self.logger
//             .info(format!("   - scanner - {}", scanner.name()));

//         self.scanners.push(scanner);
//     }

//     ///
//     ///
//     ///
//     pub fn scan(&self) -> Vec<ProductionOrder> {
//         let mut result = Vec::new();
//         for scanner in &self.scanners {
//             result.extend(scanner.scan());
//         }
//         result
//     }

//     ///
//     ///
//     ///
//     pub fn scan_as_c_string(&self) -> Result<CString, crate::Error> {
//         let result = self.scan();
//         let json_str =
//             serde_json::to_string(&result).expect("Failed to serialize scan result to JSON");
//         CString::new(json_str)
//             .map_err(|e| crate::Error::InternalLogic(format!("Failed to build CString ({:?})", e)))
//     }
// }
