use std::fmt::Debug;

use crate::{format_settings_error, log_error, log_warn, Error, InstanceSettings, Logger, Props};

#[derive(Clone)]
pub struct Settings {
    pub logger: Option<Logger>,
    pub name: String,
    pub desc: String,
    pub default_min: f64,
    pub default_max: f64,
    pub min: f64,
    pub max: f64,
    pub min_key: String,
    pub max_key: String,
}

impl Settings {
    /// Build new instance
    ///
    pub fn new<A: Into<String>, B: Into<String>, C: Into<f64>, D: Into<f64>>(
        name: A,
        desc: B,
        default_min: C,
        default_max: D,
        logger: Option<Logger>,
    ) -> Self {
        let name = name.into();
        let default_min = default_min.into();
        let default_max = default_max.into();
        Self {
            name: name.clone(),
            desc: desc.into(),
            default_min: default_min,
            default_max: default_max,
            min: default_min,
            max: default_max,
            logger: logger,
            min_key: format!("min_{}", &name),
            max_key: format!("max_{}", &name),
        }
    }

    ///
    ///
    pub fn override_with_instance_settings(
        &mut self,
        settings: &Option<InstanceSettings>,
    ) -> Result<(), Error> {
        //
        //
        if let Some(value) = settings {
            //
            //
            if value.is_object() {
                //
                //
                if let Some(map) = value.as_object() {
                    //
                    //
                    let min_value = map
                        .get(&self.min_key)
                        .and_then(|v| v.as_f64().and_then(|v| Some(v)));
                    if let Some(v) = min_value {
                        if v >= self.default_min {
                            self.min = v;
                        } else {
                            if let Some(logger) = &self.logger {
                                log_error!(
                                    logger,
                                    "{} is lower than default {} < {}",
                                    &self.min_key,
                                    v,
                                    self.default_min
                                );
                            }
                            return Err(format_settings_error!(
                                "{} is lower than default {} < {}",
                                &self.min_key,
                                v,
                                self.default_min
                            ));
                        }
                    } else {
                        if let Some(logger) = &self.logger {
                            log_warn!(
                                logger,
                                "{} is not in settings, use default value {}",
                                &self.min_key,
                                self.default_min
                            );
                        }
                    }

                    //
                    //
                    let max_value = map
                        .get(&self.max_key)
                        .and_then(|v| v.as_f64().and_then(|v| Some(v)));
                    if let Some(v) = max_value {
                        if v <= self.default_max {
                            self.max = v;
                        } else {
                            if let Some(logger) = &self.logger {
                                log_error!(
                                    logger,
                                    "{} is bigger than default {} > {}",
                                    &self.max_key,
                                    v,
                                    self.default_max
                                );
                            }
                            return Err(format_settings_error!(
                                "{} is bigger than default {} > {}",
                                &self.max_key,
                                v,
                                self.default_max
                            ));
                        }
                    } else {
                        if let Some(logger) = &self.logger {
                            log_warn!(
                                logger,
                                "{} is not in settings, use default value {}",
                                &self.max_key,
                                self.default_max
                            );
                        }
                    }
                }
            } else {
                if let Some(logger) = &self.logger {
                    log_warn!(
                        logger,
                        "Instance settings is not an object for min/max {}, use default value {}",
                        &self.name,
                        self.default_min
                    );
                }
            }
        } else {
            if let Some(logger) = &self.logger {
                log_warn!(
                    logger,
                    "No instance settings provided for min/max {}, use default value {}",
                    &self.name,
                    self.default_min
                );
            }
        }

        Ok(())
    }

    /// Add props to props
    ///
    pub fn declare(&self, props: &mut Props) {
        props.add_number_prop(
            &self.min_key,
            format!("Minimal {}", self.desc),
            self.default_min,
        );
        props.add_number_prop(
            &self.max_key,
            format!("Maximal {}", self.desc),
            self.default_max,
        );
    }
}

impl Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Settings")
            .field("name", &self.name)
            .field("desc", &self.desc)
            .field("default_min", &self.default_min)
            .field("default_max", &self.default_max)
            .field("min", &self.min)
            .field("max", &self.max)
            .field("min_key", &self.min_key)
            .field("max_key", &self.max_key)
            .finish()
    }
}
