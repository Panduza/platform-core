use super::{CallbackEntry, CallbackId};
use crate::Error;
use crate::Logger;
use crate::Notification;
use panduza::fbs::GenericBuffer;
use panduza::task_monitor::NamedTaskHandle;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use zenoh::Session;

/// Generic attribute implementation that can work with any buffer type that implements GenericBuffer
#[derive(Clone)]
pub struct GenericAttributeServer<B: GenericBuffer> {
    /// Local logger
    logger: Logger,

    /// Global Session
    session: Session,

    /// Async callbacks storage
    callbacks: Arc<Mutex<HashMap<CallbackId, CallbackEntry<B>>>>,

    /// Next callback ID
    next_callback_id: Arc<Mutex<CallbackId>>,

    /// Command topic
    cmd_topic: String,

    /// Channel to send notifications
    notification_channel: Sender<Notification>,

    /// Current value
    current_value: Arc<Mutex<Option<B>>>,
}

impl<B: GenericBuffer> GenericAttributeServer<B> {
    /// Logger getter
    ///
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    ///
    ///
    pub async fn new(
        session: Session,
        topic: String,
        task_monitor_sender: Sender<NamedTaskHandle>,
        notification_channel: Sender<Notification>,
    ) -> Self {
        // Initialize async callbacks storage
        let callbacks = Arc::new(Mutex::new(HashMap::<CallbackId, CallbackEntry<B>>::new()));

        //
        //
        let query_value = Arc::new(Mutex::new(None));

        // create a queryable to get value at initialization
        //
        let topic_clone = topic.clone();
        let session_clone = session.clone();
        let query_value_clone = query_value.clone();
        let handle = tokio::spawn(async move {
            let queryable = session_clone
                .declare_queryable(format!("{}/att", topic_clone.clone()))
                .await
                .unwrap();

            while let Ok(query) = queryable.recv_async().await {
                // let value = query_value_clone.lock().unwrap().clone(); // Clone the value
                // let pyl = Bytes::from(serde_json::to_string(&value).unwrap());
                // query
                //     .reply(format!("{}/att", topic_clone.clone()), pyl)
                //     .await
                //     .unwrap();
            }
            Ok(())
        });
        task_monitor_sender
            .send((format!("{}/ATT/QUERY", &topic), handle))
            .await
            .unwrap();

        //
        let cmd_topic = format!("{}/cmd", &topic);
        let cmd_subscriber = session.declare_subscriber(&cmd_topic).await.unwrap();

        tokio::spawn({
            let callbacks = callbacks.clone();

            // TODO => push this into a specific function for more readability
            async move {
                while let Ok(sample) = cmd_subscriber.recv_async().await {
                    // Create Buffer from the received zbytes
                    let buffer = B::from_zbytes(sample.payload().clone());

                    // Trigger all async callbacks
                    let callbacks_map = callbacks.lock().await;
                    let mut futures = Vec::new();

                    for (_id, callback_entry) in callbacks_map.iter() {
                        // Check condition if present
                        let should_trigger = if let Some(condition) = &callback_entry.condition {
                            condition(&buffer)
                        } else {
                            true
                        };

                        if should_trigger {
                            futures.push((callback_entry.callback)(buffer.clone()));
                        }
                    }

                    // Drop the lock before awaiting futures
                    drop(callbacks_map);

                    // Execute all callbacks concurrently
                    futures::future::join_all(futures).await;
                }
            }
        });

        //
        //
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            session: session,
            callbacks: callbacks,
            next_callback_id: Arc::new(Mutex::new(0)),
            cmd_topic: cmd_topic,
            notification_channel: notification_channel,
            current_value: query_value.clone(),
        }
    }

    // /// Create a new instance
    // pub async fn new(session: Session) -> Self {
    //     // Initialize async callbacks storage
    //     let callbacks = Arc::new(Mutex::new(HashMap::<CallbackId, CallbackEntry<B>>::new()));

    //     // Trigger the callback mechanism on message reception
    //     let att_topic = format!("{}/att", &metadata.topic);
    //     let subscriber = session.declare_subscriber(&att_topic).await.unwrap();
    //     let last_value = Arc::new(Mutex::new(None));

    //     tokio::spawn({
    //         let callbacks = callbacks.clone();
    //         let last_value = last_value.clone();
    //         async move {
    //             while let Ok(sample) = subscriber.recv_async().await {
    //                 // Create Buffer from the received zbytes
    //                 let buffer = B::from_zbytes(sample.payload().clone());

    //                 // Update the last received value
    //                 {
    //                     let mut last = last_value.lock().await;
    //                     *last = Some(buffer.clone());
    //                 }

    //                 // Trigger all async callbacks
    //                 let callbacks_map = callbacks.lock().await;
    //                 let mut futures = Vec::new();

    //                 for (_id, callback_entry) in callbacks_map.iter() {
    //                     // Check condition if present
    //                     let should_trigger = if let Some(condition) = &callback_entry.condition {
    //                         condition(&buffer)
    //                     } else {
    //                         true
    //                     };

    //                     if should_trigger {
    //                         futures.push((callback_entry.callback)(buffer.clone()));
    //                     }
    //                 }

    //                 // Drop the lock before awaiting futures
    //                 drop(callbacks_map);

    //                 // Execute all callbacks concurrently
    //                 futures::future::join_all(futures).await;
    //             }
    //         }
    //     });

    //     // Wait for the first message if mode is not WriteOnly
    //     if metadata.mode != AttributeMode::WriteOnly {
    //         let query = session.get(&att_topic).await.unwrap();
    //         let result = query.recv_async().await.unwrap();
    //         let buffer = B::from_zbytes(result.result().unwrap().payload().clone());
    //         let mut last = last_value.lock().await;
    //         *last = Some(buffer);
    //     }

    //     // Create the command topic
    //     let cmd_topic = format!("{}/cmd", &metadata.topic);

    //     // Return attribute
    //     Self {
    //         session,
    //         // metadata,
    //         callbacks,
    //         next_callback_id: Arc::new(Mutex::new(0)),
    //         cmd_topic,
    //         last_value,
    //     }
    // }

    ///
    ///
    pub async fn set<T>(&self, value: T) -> Result<(), Error>
    where
        T: Into<B>,
    {
        let buffer: B = value.into();

        // Send the command
        self.session
            .put(&self.cmd_topic, buffer.to_zbytes())
            .await
            .unwrap();

        // update the current queriable value
        *self.current_value.lock().await = Some(buffer);

        Ok(())
    }

    // /// Send command and do not wait for validation
    // pub async fn shoot<T>(&mut self, value: T)
    // where
    //     T: Into<B>,
    // {
    //     let buffer: B = value.into();
    //     let publisher = self
    //         .session
    //         .declare_publisher(&self.cmd_topic)
    //         .await
    //         .unwrap();

    //     publisher.put(buffer.to_zbytes()).await.unwrap();
    // }

    // /// Send command and wait for validation
    // pub async fn set<T>(&mut self, value: T) -> Result<(), String>
    // where
    //     T: Into<B> + Clone,
    //     B: PartialEq,
    // {
    //     let buffer: B = value.into();
    //     let expected_buffer = buffer.clone();

    //     self.shoot(buffer).await;

    //     // Wait for the value to be confirmed
    //     self.wait_for_value(
    //         move |received_buffer| *received_buffer == expected_buffer,
    //         Some(std::time::Duration::from_secs(5)),
    //     )
    //     .await?;

    //     Ok(())
    // }

    // /// Get last received value
    // pub async fn get(&self) -> Option<B> {
    //     let last = self.last_value.lock().await;
    //     last.clone()
    // }

    // /// Wait for a specific value with optional timeout
    // pub async fn wait_for_value<F>(
    //     &self,
    //     condition: F,
    //     timeout: Option<std::time::Duration>,
    // ) -> Result<B, String>
    // where
    //     F: Fn(&B) -> bool + Send + Sync + 'static,
    // {
    //     // Use a broadcast channel to avoid the move issue
    //     let (tx, mut rx) = tokio::sync::broadcast::channel(1);

    //     // Add temporary callback
    //     let callback_id = self
    //         .add_callback(
    //             move |buffer: B| {
    //                 let buffer_clone = buffer.clone();
    //                 let tx_clone = tx.clone();
    //                 Box::pin(async move {
    //                     let _ = tx_clone.send(buffer_clone);
    //                 })
    //             },
    //             Some(condition),
    //         )
    //         .await;

    //     let result = if let Some(duration) = timeout {
    //         tokio::time::timeout(duration, rx.recv()).await
    //     } else {
    //         // No timeout: wait indefinitely
    //         Ok(rx.recv().await)
    //     };

    //     // Remove the callback
    //     self.remove_callback(callback_id).await;

    //     match result {
    //         Ok(Ok(buffer)) => Ok(buffer),
    //         Ok(Err(_)) => Err("Channel closed unexpectedly".to_string()),
    //         Err(_) => Err("Timeout waiting for value".to_string()),
    //     }
    // }

    /// Add an async callback that will be triggered when receiving buffer messages
    /// Optionally, a condition can be provided to filter when the callback is triggered
    pub async fn add_callback<F, C>(&self, callback: F, condition: Option<C>) -> CallbackId
    where
        F: Fn(B) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
        C: Fn(&B) -> bool + Send + Sync + 'static,
    {
        let mut callbacks = self.callbacks.lock().await;
        let mut next_id = self.next_callback_id.lock().await;

        let callback_id = *next_id;
        *next_id += 1;

        let callback_entry = CallbackEntry {
            callback: Box::new(callback),
            condition: condition.map(|c| Box::new(c) as Box<dyn Fn(&B) -> bool + Send + Sync>),
        };

        callbacks.insert(callback_id, callback_entry);
        callback_id
    }

    /// Remove an async callback by its ID
    pub async fn remove_callback(&self, callback_id: CallbackId) -> bool {
        let mut callbacks = self.callbacks.lock().await;
        callbacks.remove(&callback_id).is_some()
    }

    /// Clear all async callbacks
    pub async fn clear_callbacks(&self) {
        let mut callbacks = self.callbacks.lock().await;
        callbacks.clear();
    }

    /// Get the number of registered async callbacks
    pub async fn callback_count(&self) -> usize {
        let callbacks = self.callbacks.lock().await;
        callbacks.len()
    }

    // /// Get the command topic
    // pub fn cmd_topic(&self) -> &str {
    //     &self.cmd_topic
    // }
}
