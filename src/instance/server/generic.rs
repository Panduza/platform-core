use super::{CallbackEntry, CallbackId};
use crate::AlertNotification;
use crate::Error;
use crate::Logger;
use crate::Notification;
use panduza::fbs::PanduzaBuffer;
use panduza::task_monitor::NamedTaskHandle;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use zenoh::Session;

/// Generic attribute implementation that can work with any buffer type that implements PanduzaBuffer
// #[derive(Clone)]
pub struct GenericAttributeServer<B: PanduzaBuffer> {
    /// Local logger
    logger: Logger,

    /// Global Session
    session: Session,

    /// Async callbacks storage
    callbacks: Arc<Mutex<HashMap<CallbackId, CallbackEntry<B>>>>,

    /// Next callback ID
    next_callback_id: Arc<Mutex<CallbackId>>,

    /// Attribute topic
    att_topic: String,

    /// Topic of the attribute
    topic: String,

    /// Channel to send notifications
    notification_channel: Sender<Notification>,

    /// Current value
    current_value: Arc<Mutex<B>>,
}

impl<B: PanduzaBuffer> GenericAttributeServer<B> {
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
        let cmd_topic = format!("{}/cmd", &topic);
        let att_topic = format!("{}/att", &topic);

        //
        let query_value = Arc::new(Mutex::new(B::default()));

        //
        let handle_query_processing = tokio::spawn(task_query_processing::<B>(
            session.clone(),
            att_topic.clone(),
            query_value.clone(),
        ));

        //
        task_monitor_sender
            .send((format!("{}/ATT/QRY", &topic), handle_query_processing))
            .await
            .unwrap();

        //
        let handle_command_processing = tokio::spawn(task_command_processing::<B>(
            session.clone(),
            cmd_topic.clone(),
            callbacks.clone(),
        ));

        //
        task_monitor_sender
            .send((format!("{}/CMD/SUB", &topic), handle_command_processing))
            .await
            .unwrap();

        //
        Self {
            logger: Logger::new_for_attribute_from_topic(topic.clone()),
            session: session,
            callbacks: callbacks,
            next_callback_id: Arc::new(Mutex::new(0)),
            att_topic: att_topic,
            topic: topic,
            notification_channel: notification_channel,
            current_value: query_value.clone(),
        }
    }

    ///
    ///
    pub async fn set<T>(&self, value: T) -> Result<(), Error>
    where
        T: Into<B>,
    {
        let buffer: B = value.into();

        // Send the command
        self.session
            .put(&self.att_topic, buffer.clone().to_zbytes())
            .await
            .unwrap();

        // update the current queriable value
        *self.current_value.lock().await = buffer;

        Ok(())
    }

    ///
    ///
    pub async fn reply_to<T, C>(&self, command: &C, value: T)
    where
        T: Into<B>,
        C: PanduzaBuffer,
    {
        let mut buffer: B = value.into();
        buffer = buffer.with_sequence(command.sequence()).with_source(0);

        // Send the command with acknowledgment
        self.session
            .put(&self.att_topic, buffer.to_zbytes())
            .await
            .unwrap();
    }

    ///
    ///
    pub async fn trigger_alert<T: Into<String>>(&self, message: T) {
        let notification =
            Notification::Alert(AlertNotification::new(self.topic.clone(), message.into()));
        self.notification_channel.send(notification).await.unwrap();
    }

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
}

/// Task command processing function that listens for commands and triggers callbacks
///
pub async fn task_command_processing<B: PanduzaBuffer + Send + Sync + 'static>(
    session: zenoh::Session,
    cmd_topic: String,
    callbacks: std::sync::Arc<
        tokio::sync::Mutex<std::collections::HashMap<CallbackId, CallbackEntry<B>>>,
    >,
) -> Result<(), String> {
    // Declare the command subscriber
    let cmd_subscriber = session.declare_subscriber(&cmd_topic).await.unwrap();

    // Loop to receive commands asynchronously
    while let Ok(sample) = cmd_subscriber.recv_async().await {
        // Create Buffer from the received zbytes
        let buffer = B::build_from_zbytes(sample.payload().clone());

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

    Ok(())
}

/// Task query processing function that listens for queries and replies with the current value
///
pub async fn task_query_processing<B: PanduzaBuffer + Send + Sync + 'static>(
    session: zenoh::Session,
    att_topic: String,
    query_value: std::sync::Arc<tokio::sync::Mutex<B>>,
) -> Result<(), String> {
    let queryable = session
        .declare_queryable(&att_topic)
        .await
        .map_err(|e| e.to_string())?;
    while let Ok(query) = queryable.recv_async().await {
        let p = query_value.lock().await.clone().to_zbytes();
        query
            .reply(&att_topic, p)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
