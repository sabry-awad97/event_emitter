use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

mod args_macro;

type Args = Vec<Box<dyn Any + Send + Sync>>;
type Callback = Arc<dyn Fn(&Args) + Send + Sync + 'static>;
type Listeners = Arc<RwLock<HashMap<String, Vec<(usize, Callback)>>>>;

// Define the trait `IntoArgs`
pub trait IntoArgs {
    fn into_args(self) -> Args;
}

impl IntoArgs for () {
    fn into_args(self) -> Args {
        vec![]
    }
}

impl<T: Send + Sync + 'static> IntoArgs for (T,) {
    fn into_args(self) -> Args {
        vec![Box::new(self.0)]
    }
}

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
        F: Fn(&Args) + Send + Sync + 'static,
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
        F: FnOnce(&Args) + Send + Sync + 'static,
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
    pub fn emit(&self, event: &str, args: impl IntoArgs) {
        let args = args.into_args();
        let listeners = self.listeners.read();
        if let Some(callbacks) = listeners.get(event) {
            for (_, callback) in callbacks {
                callback(&args);
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
        F: Fn(&Args) + Send + Sync + 'static,
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
        F: FnOnce(&Args) + Send + Sync + 'static,
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
