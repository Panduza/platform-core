use crate::{
    log_debug_mount_end, log_debug_mount_start, model::VectorF32AccessorModel,
    template::class::trigger, Container, Error,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

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

    let data = interface.get_vectorf32_at(index).await?;
    println!("{:?}", data); // //
                            // //
                            // let tttt = TriggerableSi {
                            //     channel: 0,
                            //     att: att_data.clone(),
                            //     interface: interface.clone(),
                            // };

    // trigger::mount(top_class, Arc::new(Mutex::new(tttt))).await?;

    // data
    // class trigger
    //    single
    //    cyclic

    log_debug_mount_end!(logger);
    Ok(())
}
