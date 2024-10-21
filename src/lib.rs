use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use args_macro::{FromArgs, IntoArgs};
use log::{debug, error, info, trace, warn};

mod args_macro;

// EventHandler trait modification
pub trait EventHandler: Send + Sync {
    fn call(&self, args: &[Box<dyn Any + Send + Sync>]);
}

// Implement EventHandler for closures
impl<F> EventHandler for F
where
    F: Fn(&[Box<dyn Any + Send + Sync>]) + Send + Sync,
{
    fn call(&self, args: &[Box<dyn Any + Send + Sync>]) {
        self(args);
    }
}

type ListenerMap = Arc<Mutex<HashMap<String, Vec<Arc<dyn EventHandler>>>>>;

// EventEmitter struct
#[derive(Clone, Default)]
pub struct EventEmitter {
    listeners: ListenerMap,
}

impl EventEmitter {
    pub fn new() -> Self {
        debug!("Creating new EventEmitter");
        EventEmitter {
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn on<F, Args>(&self, event: &str, callback: F)
    where
        F: Fn(Args) + Send + Sync + 'static,
        Args: FromArgs,
    {
        debug!("Registering listener for event: '{}'", event);
        let mut listeners = match self.listeners.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                error!(
                    "Mutex poisoned while registering listener for event '{}'. Recovering.",
                    event
                );
                poisoned.into_inner()
            }
        };

        let entry = listeners.entry(event.to_string()).or_default();
        let new_listener_count = entry.len() + 1;

        let event_clone = event.to_string();
        entry.push(Arc::new(
            move |args: &[Box<dyn Any + Send + Sync>]| match Args::from_args(args) {
                Some(converted_args) => {
                    trace!(
                        "Executing callback for event '{}' with converted arguments",
                        event_clone
                    );
                    callback(converted_args);
                }
                None => warn!(
                    "Failed to convert arguments for event '{}'. Callback not executed.",
                    event_clone
                ),
            },
        ));

        info!(
            "Listener registered successfully for event '{}'. Total listeners: {}",
            event, new_listener_count
        );
    }

    pub fn emit<A>(&self, event: &str, args: A)
    where
        A: IntoArgs,
    {
        debug!("Emitting event: '{}'", event);
        let args = args.into_args();
        let listeners = match self.listeners.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                error!(
                    "Mutex poisoned while emitting event '{}'. Recovering.",
                    event
                );
                poisoned.into_inner()
            }
        };

        match listeners.get(event) {
            Some(handlers) => {
                info!("Found {} handler(s) for event '{}'", handlers.len(), event);
                for (index, handler) in handlers.iter().enumerate() {
                    trace!(
                        "Executing handler {} of {} for event '{}'",
                        index + 1,
                        handlers.len(),
                        event
                    );
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        handler.call(&args);
                    })) {
                        Ok(_) => trace!(
                            "Handler {} for event '{}' executed successfully",
                            index + 1,
                            event
                        ),
                        Err(e) => error!(
                            "Handler {} for event '{}' panicked: {:?}",
                            index + 1,
                            event,
                            e
                        ),
                    }
                }
                info!(
                    "Finished emitting event '{}'. All handlers executed.",
                    event
                );
            }
            None => {
                warn!(
                    "No handlers found for event '{}'. Event not emitted.",
                    event
                );
            }
        }
    }

    pub fn listeners(&self, event: &str) -> Vec<Arc<dyn EventHandler>> {
        debug!("Retrieving listeners for event: {}", event);
        let listeners = self.listeners.lock().unwrap();
        let handlers = listeners
            .get(event)
            .map(|handlers| handlers.to_vec())
            .unwrap_or_default();
        trace!("Found {} listeners for event: {}", handlers.len(), event);
        handlers
    }
}
