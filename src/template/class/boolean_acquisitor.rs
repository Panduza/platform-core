use crate::{
    log_debug_mount_end, log_debug_mount_start, model::BooleanAccessorModel,
    template::class::trigger, Container, Error,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::trigger::Triggerable;

#[derive(Clone)]
struct TriggerWrap<I: BooleanAccessorModel> {
    interface: I,
    index: usize,
}

#[async_trait]
impl<I: BooleanAccessorModel> Triggerable for TriggerWrap<I> {
    async fn on_trigger(&mut self) -> Result<(), Error> {
        let value = self.interface.get_boolean_at(self.index).await?;
        // self.att.set_from_f32(value as f32).await?;
        Ok(())
    }
}

///
///
pub async fn mount<A: Into<String>, C: Container, I: BooleanAccessorModel + Clone + 'static>(
    mut parent: C,
    mut interface: I,
    index: usize,
    name: A,
) -> Result<(), Error> {
    //
    //
    let mut top_class = parent
        .create_class(&name.into())
        .with_tag("boolean_acquisitor")
        .finish()
        .await;
    let logger = top_class.logger().clone();
    log_debug_mount_start!(logger);

    //
    //
    let att_data = top_class
        .create_attribute("data")
        .with_ro()
        .start_as_boolean()
        .await?;

    //
    //
    let trigger_interface = TriggerWrap {
        // channel: 0,
        // att: att_data.clone(),
        interface: interface.clone(),
        index: index,
    };
    trigger::mount(top_class, trigger_interface).await?;

    //
    log_debug_mount_end!(logger);
    Ok(())
}
