use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

type Callback = Arc<dyn Fn(&[&(dyn Any + Send + Sync)]) + Send + Sync + 'static>;
type Listeners = Arc<RwLock<HashMap<String, Vec<(usize, Callback)>>>>;

mod args_macro;

#[derive(Clone)]
pub struct EventEmitter {
    listeners: Listeners,
    next_id: Arc<AtomicUsize>,
}

impl Default for EventEmitter {
    /// Creates a new `EventEmitter`.
    fn default() -> Self {
        Self::new()
    }
}

impl EventEmitter {
    /// Creates a new `EventEmitter`.
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Registers a callback for the specified event.
    ///
    /// # Arguments
    ///
    /// * `event` - The event name to listen for.
    /// * `callback` - The callback function to be called when the event is emitted.
    ///
    /// # Returns
    ///
    /// The ID of the registered listener, which can be used to remove it later.
    pub fn on<F>(&self, event: impl Into<String>, callback: F) -> usize
    where
        F: Fn(&[&(dyn Any + Send + Sync)]) + Send + Sync + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut listeners = self.listeners.write();
        listeners
            .entry(event.into())
            .or_default()
            .push((id, Arc::new(callback)));
        id
    }

    /// Registers a callback for the specified event that will be called only once.
    ///
    /// # Arguments
    ///
    /// * `event` - The event name to listen for.
    /// * `callback` - The callback function to be called when the event is emitted.
    pub fn once<F>(&self, event: impl Into<String>, callback: F)
    where
        F: FnOnce(&[&(dyn Any + Send + Sync)]) + Send + Sync + 'static,
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
            if let Some(cb) = callback.lock().unwrap().take() {
                cb(args);
            }
        });
        id.load(Ordering::SeqCst);
    }

    /// Emits an event with the given name and arguments.
    ///
    /// # Arguments
    ///
    /// * `event` - The name of the event to emit.
    /// * `args` - The arguments to pass to the event listeners.
    pub fn emit(&self, event: &str, args: &[&(dyn Any + Send + Sync)]) {
        let listeners = self.listeners.read();
        if let Some(callbacks) = listeners.get(event) {
            for (_, callback) in callbacks {
                callback(args);
            }
        }
    }

    /// Returns a list of listeners for the specified event.
    ///
    /// # Arguments
    ///
    /// * `event` - The name of the event to get the listeners for.
    pub fn listeners(&self, event: &str) -> Vec<Callback> {
        let listeners = self.listeners.read();
        listeners
            .get(event)
            .map(|callbacks| callbacks.iter().map(|(_, cb)| Arc::clone(cb)).collect())
            .unwrap_or_default()
    }

    /// Returns the number of listeners for the specified event.
    ///
    /// # Arguments
    ///
    /// * `event` - The name of the event to get the listener count for.
    pub fn listener_count(&self, event: &str) -> usize {
        let listeners = self.listeners.read();
        listeners
            .get(event)
            .map(|callbacks| callbacks.len())
            .unwrap_or(0)
    }

    /// Returns a list of all event names.
    pub fn event_names(&self) -> Vec<String> {
        let listeners = self.listeners.read();
        listeners.keys().cloned().collect()
    }

    /// Returns the maximum number of listeners that can be registered for an event.
    pub fn get_max_listeners(&self) -> usize {
        // In this implementation, we don't have a max listeners limit
        // You can add a field to EventEmitter to store this value if needed
        usize::MAX
    }

    pub fn set_max_listeners(&mut self, _n: usize) {
        // In this implementation, we don't enforce a max listeners limit
        // You can add a field to EventEmitter to store this value if needed
    }

    /// Adds a listener to the beginning of the listeners array for the specified event.
    ///
    /// # Arguments
    ///
    /// * `event` - The event name to listen for.
    /// * `callback` - The callback function to be called when the event is emitted.
    pub fn prepend_listener<F>(&self, event: impl Into<String>, callback: F) -> usize
    where
        F: Fn(&[&(dyn Any + Send + Sync)]) + Send + Sync + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut listeners = self.listeners.write();
        listeners
            .entry(event.into())
            .or_default()
            .insert(0, (id, Arc::new(callback)));
        id
    }

    /// Adds a listener to the beginning of the listeners array for the specified event that will be called only once.
    ///
    /// # Arguments
    ///
    /// * `event` - The event name to listen for.
    /// * `callback` - The callback function to be called when the event is emitted.
    pub fn prepend_once_listener<F>(&self, event: impl Into<String>, callback: F)
    where
        F: FnOnce(&[&(dyn Any + Send + Sync)]) + Send + Sync + 'static,
    {
        let weak_self = Arc::downgrade(&Arc::new(self.clone()));
        let event = event.into();
        let event_clone = event.clone();
        let callback = Mutex::new(Some(callback));
        let id = Arc::new(AtomicUsize::new(0));
        let id_clone = Arc::clone(&id);
        self.prepend_listener(event, move |args| {
            let current_id = id_clone.load(Ordering::SeqCst);
            if let Some(strong_self) = weak_self.upgrade() {
                strong_self.remove_listener(&event_clone, current_id);
            }
            if let Some(cb) = callback.lock().unwrap().take() {
                cb(args);
            }
        });
        id.load(Ordering::SeqCst);
    }

    /// Removes a listener from the specified event.
    ///
    /// # Arguments
    ///
    /// * `event` - The event name to remove the listener from.
    /// * `id` - The ID of the listener to remove.
    pub fn remove_listener(&self, event: &str, id: usize) {
        let mut listeners = self.listeners.write();
        if let Some(callbacks) = listeners.get_mut(event) {
            callbacks.retain(|(callback_id, _)| *callback_id != id);
        }
    }

    /// Removes all listeners for the specified event.
    ///
    /// # Arguments
    ///
    /// * `event` - The event name to remove all listeners from.
    pub fn remove_listeners(&self, event: &str) {
        let mut listeners = self.listeners.write();
        listeners.remove(event);
    }

    /// Removes all listeners from the emitter.
    pub fn remove_all_listeners(&mut self) {
        let mut listeners = self.listeners.write();
        listeners.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_on_and_emit() {
        let emitter = EventEmitter::new();
        let counter = Arc::new(AtomicUsize::new(0));

        emitter.on("test_event", {
            let counter = Arc::clone(&counter);
            move |_args: &[&(dyn Any + Send + Sync)]| {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event", &[]);
        emitter.emit("test_event", &[]);

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_once() {
        let emitter = EventEmitter::new();
        let counter = Arc::new(AtomicUsize::new(0));

        emitter.once("test_event", {
            let counter = Arc::clone(&counter);
            move |_args: &[&(dyn Any + Send + Sync)]| {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event", &[]);
        emitter.emit("test_event", &[]);

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_remove_listener() {
        let emitter = EventEmitter::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let callback_id = emitter.on("test_event", {
            let counter = Arc::clone(&counter);
            move |_args: &[&(dyn Any + Send + Sync)]| {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event", &[]);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        emitter.remove_listener("test_event", callback_id);

        emitter.emit("test_event", &[]);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_listeners() {
        let emitter = EventEmitter::new();
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        emitter.on("test_event", {
            let counter = Arc::clone(&counter1);
            move |_args: &[&(dyn Any + Send + Sync)]| {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.on("test_event", {
            let counter = Arc::clone(&counter2);
            move |_args| {
                counter.fetch_add(2, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event", &[]);

        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_non_existent_event() {
        let emitter = EventEmitter::new();
        // This should not panic or cause any errors
        emitter.emit("non_existent_event", &[]);
    }

    #[test]
    fn test_remove_listeners() {
        let emitter = EventEmitter::new();
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        emitter.on("test_event", {
            let counter = Arc::clone(&counter1);
            move |_args| {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.on("test_event", {
            let counter = Arc::clone(&counter2);
            move |_args| {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event", &[]);
        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 1);

        emitter.remove_listeners("test_event");
        emitter.emit("test_event", &[]);
        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_listener_count() {
        let emitter = EventEmitter::new();
        assert_eq!(emitter.listener_count("test_event"), 0);

        emitter.on("test_event", |_args| {});
        assert_eq!(emitter.listener_count("test_event"), 1);

        emitter.on("test_event", |_args| {});
        assert_eq!(emitter.listener_count("test_event"), 2);

        emitter.remove_listeners("test_event");
        assert_eq!(emitter.listener_count("test_event"), 0);
    }

    #[test]
    fn test_event_names() {
        let emitter = EventEmitter::new();
        assert!(emitter.event_names().is_empty());

        emitter.on("event1", |_args| {});
        emitter.on("event2", |_args| {});
        emitter.on("event3", |_args| {});

        let names = emitter.event_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"event1".to_string()));
        assert!(names.contains(&"event2".to_string()));
        assert!(names.contains(&"event3".to_string()));
    }

    #[test]
    fn test_prepend_listener() {
        let emitter = EventEmitter::new();
        let order = Arc::new(Mutex::new(Vec::new()));

        emitter.on("test_event", {
            let order = Arc::clone(&order);
            move |_args| {
                order.lock().unwrap().push(2);
            }
        });

        emitter.prepend_listener("test_event", {
            let order = Arc::clone(&order);
            move |_args| {
                order.lock().unwrap().push(1);
            }
        });

        emitter.emit("test_event", &[]);

        assert_eq!(*order.lock().unwrap(), vec![1, 2]);
    }
}
