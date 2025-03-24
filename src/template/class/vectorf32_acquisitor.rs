use crate::{
    instance::server::vector_f32_v0::VectorF32AttributeServer, log_debug_mount_end,
    log_debug_mount_start, model::VectorF32AccessorModel, template::class::trigger, Container,
    Error,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::trigger::Triggerable;

#[derive(Clone)]
struct TriggerWrap<I: VectorF32AccessorModel> {
    interface: I,
    index: usize,
    att_srv: VectorF32AttributeServer,
}

#[async_trait]
impl<I: VectorF32AccessorModel> Triggerable for TriggerWrap<I> {
    async fn on_trigger(&mut self) -> Result<(), Error> {
        let value = self.interface.get_vectorf32_at(self.index).await?;
        self.att_srv.set(&value).await?;
        Ok(())
    }
}

///
///
pub async fn mount<A: Into<String>, C: Container, I: VectorF32AccessorModel + Clone + 'static>(
    mut parent: C,
    mut interface: I,
    index: usize,
    name: A,
) -> Result<(), Error> {
    //
    //
    let mut top_class = parent
        .create_class(&name.into())
        .with_tag("vectorf32_acquisitor")
        .finish()
        .await;
    let logger = top_class.logger().clone();
    log_debug_mount_start!(logger);

    //
    //
    let att_data = top_class
        .create_attribute("data")
        .with_ro()
        .start_as_vector_f32()
        .await?;

    // let data = interface.get_vectorf32_at(index).await?;
    // println!("{:?}", data);

    //
    //
    let trigger_wrap = TriggerWrap {
        // channel: 0,
        // att: att_data.clone(),
        interface: interface.clone(),
        index: index,
        att_srv: att_data,
    };
    trigger::mount(top_class, trigger_wrap, "trigger").await?;

    log_debug_mount_end!(logger);
    Ok(())
}
