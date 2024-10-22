use std::borrow::Cow;

pub trait IntoEvent {
    fn into_event(self) -> Cow<'static, str>;
}

// Implement IntoEvent for common types
impl IntoEvent for &str {
    fn into_event(self) -> Cow<'static, str> {
        Cow::Owned(self.to_string())
    }
}

impl IntoEvent for String {
    fn into_event(self) -> Cow<'static, str> {
        Cow::Owned(self)
    }
}

impl IntoEvent for Cow<'static, str> {
    fn into_event(self) -> Cow<'static, str> {
        self
    }
}

// Add a macro for easy event creation
#[macro_export]
macro_rules! event {
    ($($arg:expr),*) => {{
        let mut event = String::new();
        $(
            event.push_str(&$arg.to_string());
            event.push('_');
        )*
        event.pop(); // Remove the trailing underscore
        Cow::Owned(event)
    }};
}

// Add tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_event() {
        assert_eq!(event!("hello"), "hello");
        assert_eq!(event!("hello", "world", 42), "hello_world_42");
    }
}
