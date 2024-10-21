use std::any::Any;

type Args = Vec<Box<dyn Any + Send + Sync>>;

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

impl<T: Send + Sync + 'static + Clone, U: Send + Sync + 'static + Clone> IntoArgs for (T, U) {
    fn into_args(self) -> Args {
        vec![Box::new(self.0), Box::new(self.1)]
    }
}

// Define the trait `FromArgs`
pub trait FromArgs: Sized {
    fn from_args(args: &Args) -> Option<Self>;
}

impl FromArgs for () {
    fn from_args(_args: &Args) -> Option<Self> {
        Some(())
    }
}

impl<T: Send + Sync + 'static + Clone> FromArgs for (T,) {
    fn from_args(args: &Args) -> Option<Self> {
        args.first()
            .and_then(|arg| arg.downcast_ref::<T>())
            .cloned()
            .map(|t| (t,))
    }
}

impl<T: Send + Sync + 'static + Clone, U: Send + Sync + 'static + Clone> FromArgs for (T, U) {
    fn from_args(args: &Args) -> Option<Self> {
        if args.len() < 2 {
            return None;
        }
        let t = args[0].downcast_ref::<T>().cloned()?;
        let u = args[1].downcast_ref::<U>().cloned()?;
        Some((t, u))
    }
}

// Helper macros for counting tuple elements
macro_rules! count {
    ($($T:ident),+) => {
        <[()]>::len(&[$(replace_expr!($T ())),+])
    };
}

macro_rules! count_index {
    ($T:ident) => {
        replace_expr!($T 0)
    };
    ($T1:ident, $($T:ident),+) => {
        1 + count_index!($($T),+)
    };
}

// Helper macro to ignore the expression and substitute a replacement
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_macro() {
        assert_eq!(count!(A), 1);
        assert_eq!(count!(A, B), 2);
        assert_eq!(count!(A, B, C), 3);
        assert_eq!(count!(A, B, C, D, E), 5);
    }

    #[test]
    fn test_count_index_macro() {
        assert_eq!(count_index!(A), 0);
        assert_eq!(count_index!(A, B), 1);
        assert_eq!(count_index!(A, B, C), 2);
        assert_eq!(count_index!(A, B, C, D), 3);
    }

    #[test]
    fn test_replace_expr_macro() {
        assert_eq!(replace_expr!(any_token 42), 42);
        assert_eq!(replace_expr!(ignored_expr "hello"), "hello");
    }

    #[test]
    fn test_complex_count() {
        assert_eq!(count!(A, B, C, D, E, F, G, H, I, J), 10);
    }

    #[test]
    fn test_complex_count_index() {
        assert_eq!(count_index!(A, B, C, D, E, F, G, H, I, J), 9);
    }
}
