use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use args_macro::{FromArgs, IntoArgs};

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
        EventEmitter {
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn on<F, Args>(&self, event: &str, callback: F)
    where
        F: Fn(Args) + Send + Sync + 'static,
        Args: FromArgs,
    {
        let mut listeners = self.listeners.lock().unwrap();
        listeners
            .entry(event.to_string())
            .or_default()
            .push(Arc::new(move |args: &[Box<dyn Any + Send + Sync>]| {
                if let Some(converted_args) = Args::from_args(args) {
                    callback(converted_args);
                }
            }));
    }

    pub fn emit<A>(&self, event: &str, args: A)
    where
        A: IntoArgs,
    {
        let args = args.into_args();
        let listeners = self.listeners.lock().unwrap();
        if let Some(handlers) = listeners.get(event) {
            for handler in handlers {
                handler.call(&args);
            }
        }
    }

    pub fn listeners(&self, event: &str) -> Vec<Arc<dyn EventHandler>> {
        let listeners = self.listeners.lock().unwrap();
        listeners
            .get(event)
            .map(|handlers| handlers.to_vec())
            .unwrap_or_default()
    }
}
