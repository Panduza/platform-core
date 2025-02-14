use crate::Topic;

/// Generic way to build logs on the platform
///
#[derive(Clone)]
pub struct Logger {
    pub class: String,
    pub i1: String,
    pub i2: String,
    pub i3: String,
    pub plugin: String,
}
impl Logger {
    /// Call this if the logger is a for a plugin
    ///
    pub fn set_plugin<A: Into<String>>(&mut self, text: A) {
        self.plugin = text.into();
    }

    /// Get the plugin name
    ///
    pub fn get_plugin(&self) -> String {
        self.plugin.clone()
    }

    /// Create a new logger
    ///
    pub fn new<A: Into<String>, B: Into<String>, C: Into<String>, D: Into<String>>(
        class: A,
        i1: B,
        i2: C,
        i3: D,
    ) -> Logger {
        return Logger {
            class: class.into(),
            i1: i1.into(),
            i2: i2.into(),
            i3: i3.into(),
            plugin: String::new(),
        };
    }

    /// Create a logger configured for platform from its name
    ///
    pub fn new_for_platform() -> Self {
        Self::new("Platform", "", "", "")
    }

    /// Create a logger configured for runtime from its name
    ///
    pub fn new_for_runtime() -> Self {
        Self::new("Runtime", "", "", "")
    }

    /// Create a logger configured for instance from its name
    ///
    pub fn new_for_instance<A: Into<String>>(name: A) -> Self {
        Self::new("Instance", name.into(), "", "")
    }

    /// Create a logger configured for drivers
    ///
    pub fn new_for_driver<A: Into<String>, B: Into<String>>(phy: A, prot: B) -> Self {
        Self::new("Driver", phy.into(), prot.into(), "")
    }

    /// Create a logger configured for isolated instance (ex: function without object)
    ///
    pub fn new_isolated<A: Into<String>>(name: A) -> Self {
        Self::new("Isolated", name.into(), "", "")
    }

    /// Create a logger configured for attribute from its topic
    ///
    pub fn new_for_attribute_from_topic<A: Into<String>>(topic: A) -> Self {
        let topic_obj = Topic::from_string(topic.into(), true);
        Self::new(
            "Attribute",
            topic_obj.instance_name(),
            topic_obj.class_stack_name(),
            topic_obj.leaf_name().unwrap_or(&"".to_string()),
        )
    }

    /// Create a new class logger from this logger (supposed instance logger)
    ///
    pub fn new_for_class<B: Into<String>>(&self, topic: B) -> Self {
        let topic_obj = Topic::from_string(topic.into(), false);
        let mut new_logger = Self::new("Class", &self.i1, topic_obj.class_stack_name(), "");
        new_logger.set_plugin(&self.plugin);
        new_logger
    }

    /// Create a new attribute logger from this logger (supposed instance logger)
    ///
    pub fn new_for_attribute<B: Into<String>>(&self, class: Option<String>, attribute: B) -> Self {
        //
        // Extract class name
        let class_name = if let Some(c) = class {
            c.into()
        } else {
            "".to_string()
        };

        //
        // Create the logger
        let mut new_logger = Self::new("Attribute", &self.i1, class_name, attribute.into());
        new_logger.set_plugin(&self.plugin);
        new_logger
    }

    pub fn error<A: Into<String>>(&self, text: A) {
        tracing::error!(
            class = self.class,
            i1 = self.i1,
            i2 = self.i2,
            i3 = self.i3,
            plugin = self.plugin,
            "{}",
            text.into()
        );
    }

    pub fn warn<A: Into<String>>(&self, text: A) {
        tracing::warn!(
            class = self.class,
            i1 = self.i1,
            i2 = self.i2,
            i3 = self.i3,
            plugin = self.plugin,
            "{}",
            text.into()
        );
    }

    pub fn info<A: Into<String>>(&self, text: A) {
        tracing::info!(
            class = self.class,
            i1 = self.i1,
            i2 = self.i2,
            i3 = self.i3,
            plugin = self.plugin,
            "{}",
            text.into()
        );
    }

    pub fn debug<A: Into<String>>(&self, text: A) {
        tracing::debug!(
            class = self.class,
            i1 = self.i1,
            i2 = self.i2,
            i3 = self.i3,
            plugin = self.plugin,
            "{}",
            text.into()
        );
    }

    pub fn trace<A: Into<String>>(&self, text: A) {
        tracing::trace!(
            class = self.class,
            i1 = self.i1,
            i2 = self.i2,
            i3 = self.i3,
            plugin = self.plugin,
            "{}",
            text.into()
        );
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct FactoryLogger {
    base: Logger,
}
impl FactoryLogger {
    pub fn new() -> FactoryLogger {
        FactoryLogger {
            base: Logger::new("Factory", "", "", ""),
        }
    }
    pub fn info<A: Into<String>>(&self, text: A) {
        self.base.info(text);
    }
}

#[macro_export]
macro_rules! log_error {
    ($logger:expr , $($arg:tt)*) => {
        $logger.error(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($logger:expr , $($arg:tt)*) => {
        $logger.warn(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($logger:expr , $($arg:tt)*) => {
        $logger.info(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($logger:expr , $($arg:tt)*) => {
        $logger.debug(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_trace {
    ($logger:expr , $($arg:tt)*) => {
        $logger.trace(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info_mount_start {
    ($logger:expr) => {
        $logger.info("Mounting...")
    };
}

#[macro_export]
macro_rules! log_info_mount_end {
    ($logger:expr) => {
        $logger.info("Mounting -> OK")
    };
}

#[macro_export]
macro_rules! log_debug_mount_start {
    ($logger:expr) => {
        $logger.debug("Mounting...")
    };
}

#[macro_export]
macro_rules! log_debug_mount_end {
    ($logger:expr) => {
        $logger.debug("Mounting -> OK")
    };
}
