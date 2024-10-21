use args_macro::{FromArgs, IntoArgs};
use log::{debug, error, info, trace, warn};
use parking_lot::{Mutex, RwLock};
use smallvec::SmallVec;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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

type ListenerMap = Arc<RwLock<HashMap<String, SmallVec<[(usize, Arc<dyn EventHandler>); 4]>>>>;

// EventEmitter struct
#[derive(Clone, Default)]
pub struct EventEmitter {
    listeners: ListenerMap,
    next_id: Arc<AtomicUsize>,
}

impl EventEmitter {
    pub fn new() -> Self {
        debug!("Creating new EventEmitter");
        EventEmitter {
            listeners: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn once<F, Args>(&self, event: impl Into<String>, callback: F) -> usize
    where
        F: Fn(Args) + Send + Sync + 'static,
        Args: FromArgs,
    {
        let weak_self = Arc::downgrade(&Arc::new(self.clone()));
        let event = event.into();
        let event_clone = event.clone();
        let callback = Mutex::new(Some(callback));
        let id = Arc::new(AtomicUsize::new(0));
        let id_clone = Arc::clone(&id);
        self.on(event, move |args| {
            let current_id = id_clone.load(Ordering::SeqCst);
            if let Some(strong_self) = weak_self.upgrade() {
                strong_self.remove_listener(&event_clone, current_id);
            }
            if let Some(cb) = callback.lock().take() {
                cb(args);
            }
        });
        id.load(Ordering::SeqCst)
    }

    pub fn on<F, Args>(&self, event: impl Into<String>, callback: F) -> usize
    where
        F: Fn(Args) + Send + Sync + 'static,
        Args: FromArgs,
    {
        let event: String = event.into();
        debug!("Registering listener for event: '{}'", event);
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut listeners = self.listeners.write();
        let entry = listeners.entry(event.clone()).or_default();
        let new_listener_count = entry.len() + 1;

        let event_clone = event.clone();
        entry.push((
            id,
            Arc::new(
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
            ) as Arc<dyn EventHandler>,
        ));

        info!(
            "Listener registered successfully for event '{}'. Total listeners: {}",
            event, new_listener_count
        );
        id
    }

    pub fn emit<A>(&self, event: &str, args: A)
    where
        A: IntoArgs,
    {
        debug!("Emitting event: '{}'", event);
        let args = args.into_args();
        let listeners = self.listeners.read();

        if let Some(handlers) = listeners.get(event) {
            info!("Found {} handler(s) for event '{}'", handlers.len(), event);
            for (index, (_, handler)) in handlers.iter().enumerate() {
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
        } else {
            warn!(
                "No handlers found for event '{}'. Event not emitted.",
                event
            );
        }
    }

    pub fn listeners_count(&self, event: &str) -> usize {
        debug!("Retrieving listeners count for event: {}", event);
        let listeners = self.listeners.read();
        listeners.get(event).cloned().unwrap_or_default().len()
    }

    pub fn remove_listeners(&self) {
        debug!("Removing all listeners");
        let mut listeners = self.listeners.write();
        listeners.clear();
    }

    pub fn remove_listener(&self, event: &str, id: usize) {
        let mut listeners = self.listeners.write();
        if let Some(callbacks) = listeners.get_mut(event) {
            callbacks.retain(|(callback_id, _)| *callback_id != id);
        }
    }
}
