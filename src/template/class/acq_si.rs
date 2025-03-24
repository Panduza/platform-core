use crate::{
    log_debug_mount_end, log_debug_mount_start, std::class::trigger, Container, Error, SiAttServer,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::trigger::Triggerable;

#[async_trait]
///
///
pub trait SiDataReader: Sync + Send {
    ///
    ///
    async fn read_data(&mut self, channel: usize) -> Result<f64, Error>;
}

#[derive(Clone)]
struct TriggerableSi<I: SiDataReader> {
    channel: usize,
    att: SiAttServer,
    interface: Arc<Mutex<I>>,
}

#[async_trait]
impl<I: SiDataReader> Triggerable for TriggerableSi<I> {
    async fn on_trigger(&mut self) -> Result<(), Error> {
        let value = self.interface.lock().await.read_data(self.channel).await?;
        self.att.set_from_f32(value as f32).await?;
        Ok(())
    }
}

///
///
pub async fn mount<A: Into<String>, N: Into<String>, C: Container, I: SiDataReader + 'static>(
    name: A,
    unit: N,
    min: f64,
    max: f64,
    decimals: usize,
    mut parent: C,
    interface: Arc<Mutex<I>>,
) -> Result<(), Error> {
    //
    //
    let mut class_acq_si = parent
        .create_class(&name.into())
        .with_tag("acq_si")
        .finish()
        .await;
    let logger = class_acq_si.logger().clone();
    log_debug_mount_start!(logger);

    //
    //
    let att_data = class_acq_si
        .create_attribute("data")
        .with_ro()
        .finish_as_si(unit, min, max, decimals)
        .await?;

    //
    //
    let tttt = TriggerableSi {
        channel: 0,
        att: att_data.clone(),
        interface: interface.clone(),
    };

    trigger::mount(class_acq_si, Arc::new(Mutex::new(tttt))).await?;

    // data
    // class trigger
    //    single
    //    cyclic

    log_debug_mount_end!(logger);
    Ok(())
}
