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

// Implement IntoEvent for tuples of strings and integers
impl<T: ToString> IntoEvent for (T, i32) {
    fn into_event(self) -> Cow<'static, str> {
        Cow::Owned(format!("{}_{}", self.0.to_string(), self.1))
    }
}
