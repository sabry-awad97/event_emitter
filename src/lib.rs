use args_macro::{FromArgs, IntoArgs};
use event::IntoEvent;
use log::{debug, error, info, trace, warn};
use parking_lot::{Mutex, RwLock};
use smallvec::SmallVec;
use std::any::Any;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

mod args_macro;
mod event;

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
#[derive(Clone)]
pub struct EventEmitter {
    listeners: ListenerMap,
    next_id: Arc<AtomicUsize>,
    max_listeners: NonZeroUsize,
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventEmitter {
    pub fn new() -> Self {
        Self::with_capacity(NonZeroUsize::new(10).unwrap())
    }

    pub fn with_capacity(max_listeners: NonZeroUsize) -> Self {
        debug!(
            "Creating new EventEmitter with max_listeners: {}",
            max_listeners
        );
        EventEmitter {
            listeners: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(AtomicUsize::new(1)), // Start from 1 to avoid 0 as a valid ID
            max_listeners,
        }
    }

    pub fn once<F, Args, E>(&self, event: E, callback: F) -> usize
    where
        F: Fn(Args) + Send + Sync + 'static,
        Args: FromArgs,
        E: IntoEvent,
    {
        let event = event.into_event();
        let weak_self = Arc::downgrade(&Arc::new(self.clone()));
        let callback = Mutex::new(Some(callback));
        let id = Arc::new(AtomicUsize::new(0));
        let id_clone = Arc::clone(&id);
        self.on(event.clone(), move |args| {
            let current_id = id_clone.load(Ordering::SeqCst);
            if let Some(strong_self) = weak_self.upgrade() {
                strong_self.remove_listener(&*event, current_id);
            }
            if let Some(cb) = callback.lock().take() {
                cb(args);
            }
        });
        id.load(Ordering::SeqCst)
    }

    pub fn on<F, Args, E>(&self, event: E, callback: F) -> usize
    where
        F: Fn(Args) + Send + Sync + 'static,
        Args: FromArgs,
        E: IntoEvent,
    {
        let event = event.into_event();
        debug!("Registering listener for event: '{}'", event);
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut listeners = self.listeners.write();
        let entry = listeners.entry(event.to_string()).or_default();
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
            "Listener registered successfully for event '{}'. Total listeners: {}. Listener ID: {}",
            event, new_listener_count, id
        );
        id
    }

    pub fn emit<A, E>(&self, event: E, args: A)
    where
        A: IntoArgs,
        E: IntoEvent,
    {
        let event = event.into_event();
        debug!("Emitting event: '{}'", event);
        let args = args.into_args();
        let listeners = self.listeners.read();

        if let Some(handlers) = listeners.get(event.as_ref()) {
            info!("Found {} handler(s) for event '{}'", handlers.len(), event);
            for (index, (id, handler)) in handlers.iter().enumerate() {
                trace!(
                    "Executing handler {} (ID: {}) of {} for event '{}'",
                    index + 1,
                    id,
                    handlers.len(),
                    event
                );
                if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    handler.call(&args);
                })) {
                    error!(
                        "Handler {} (ID: {}) for event '{}' panicked: {:?}",
                        index + 1,
                        id,
                        event,
                        e
                    );
                } else {
                    trace!(
                        "Handler {} (ID: {}) for event '{}' executed successfully",
                        index + 1,
                        id,
                        event
                    );
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

    pub fn listeners_count<E: IntoEvent>(&self, event: E) -> usize {
        let event = event.into_event();
        debug!("Retrieving listeners count for event: {}", event);
        let listeners = self.listeners.read();
        let count = listeners.get(event.as_ref()).map_or(0, |v| v.len());
        trace!("Found {} listener(s) for event '{}'", count, event);
        count
    }

    pub fn remove_listeners(&self) {
        debug!("Removing all listeners");
        let mut listeners = self.listeners.write();
        let total_removed = listeners.len();
        listeners.clear();
        info!("Removed all listeners. Total removed: {}", total_removed);
    }

    pub fn remove_listener<E: IntoEvent>(&self, event: E, id: usize) {
        let event = event.into_event();
        debug!("Removing listener with ID {} for event '{}'", id, event);
        let mut listeners = self.listeners.write();
        if let Some(callbacks) = listeners.get_mut(event.as_ref()) {
            let initial_count = callbacks.len();
            callbacks.retain(|(callback_id, _)| *callback_id != id);
            let removed_count = initial_count - callbacks.len();
            if removed_count > 0 {
                info!(
                    "Removed listener with ID {} for event '{}'. {} listener(s) remaining.",
                    id,
                    event,
                    callbacks.len()
                );
            } else {
                warn!("No listener found with ID {} for event '{}'", id, event);
            }
        } else {
            warn!("No listeners found for event '{}'", event);
        }
    }

    pub fn set_max_listeners(&mut self, max: NonZeroUsize) {
        debug!("Setting max listeners to {}", max);
        self.max_listeners = max;
    }

    pub fn get_max_listeners(&self) -> NonZeroUsize {
        self.max_listeners
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_event_emitter() {
        let emitter = EventEmitter::new();
        let (tx, rx) = mpsc::channel();

        emitter.on("test", move |args: (i32,)| {
            tx.send(args.0).unwrap();
        });

        emitter.emit("test", (42,));
        assert_eq!(rx.recv().unwrap(), 42);
    }

    #[test]
    fn test_multiple_listeners() {
        let emitter = EventEmitter::new();
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        emitter.on("multi", move |args: (String,)| {
            tx1.send(args.0.clone()).unwrap();
        });
        emitter.on("multi", move |args: (String,)| {
            tx2.send(args.0.clone()).unwrap();
        });

        emitter.emit("multi", ("Hello".to_string(),));

        assert_eq!(rx1.recv().unwrap(), "Hello");
        assert_eq!(rx2.recv().unwrap(), "Hello");
    }

    #[test]
    fn test_once() {
        let emitter = EventEmitter::new();
        let (tx, rx) = mpsc::channel();

        emitter.once("once", move |args: (i32,)| {
            tx.send(args.0).unwrap();
        });

        emitter.emit("once", (1,));
        emitter.emit("once", (2,));

        assert_eq!(rx.recv().unwrap(), 1);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_remove_listener() {
        let emitter = EventEmitter::new();
        let (tx, rx) = mpsc::channel();

        let id = emitter.on("remove", move |args: (i32,)| {
            tx.send(args.0).unwrap();
        });

        emitter.emit("remove", (1,));
        assert_eq!(rx.recv().unwrap(), 1);

        emitter.remove_listener("remove", id);
        emitter.emit("remove", (2,));

        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_remove_all_listeners() {
        let emitter = EventEmitter::new();
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        emitter.on("event1", move |args: (i32,)| {
            tx1.send(args.0).unwrap();
        });
        emitter.on("event2", move |args: (i32,)| {
            tx2.send(args.0).unwrap();
        });

        emitter.remove_listeners();

        emitter.emit("event1", (1,));
        emitter.emit("event2", (2,));

        assert!(rx1.try_recv().is_err());
        assert!(rx2.try_recv().is_err());
    }

    #[test]
    fn test_listeners_count() {
        let emitter = EventEmitter::new();

        assert_eq!(emitter.listeners_count("test"), 0);

        emitter.on("test", |_: ()| {});
        assert_eq!(emitter.listeners_count("test"), 1);

        emitter.on("test", |_: ()| {});
        assert_eq!(emitter.listeners_count("test"), 2);

        emitter.on("other", |_: ()| {});
        assert_eq!(emitter.listeners_count("test"), 2);
        assert_eq!(emitter.listeners_count("other"), 1);
    }

    #[test]
    fn test_max_listeners() {
        let mut emitter = EventEmitter::new();
        assert_eq!(emitter.get_max_listeners(), NonZeroUsize::new(10).unwrap());

        emitter.set_max_listeners(NonZeroUsize::new(5).unwrap());
        assert_eq!(emitter.get_max_listeners(), NonZeroUsize::new(5).unwrap());
    }

    #[test]
    fn test_thread_safety() {
        let emitter = Arc::new(EventEmitter::new());
        let (tx, rx) = mpsc::channel();

        let emitter_clone = Arc::clone(&emitter);
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            emitter_clone.emit("threaded", ("Hello from thread".to_string(),));
        });

        emitter.on("threaded", move |args: (String,)| {
            tx.send(args.0).unwrap();
        });

        handle.join().unwrap();
        assert_eq!(rx.recv().unwrap(), "Hello from thread");
    }
}
