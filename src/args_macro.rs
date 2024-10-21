use std::any::Any;

type AnyArgs = Vec<Box<dyn Any + Send + Sync>>;

// Trait for converting from boxed Any arguments
pub trait FromArgs: Sized {
    fn from_args(args: &[Box<dyn Any + Send + Sync>]) -> Option<Self>;
}

// Trait for converting into boxed Any arguments
pub trait IntoArgs {
    fn into_args(self) -> AnyArgs;
}

// Macro to implement FromArgs and IntoArgs for tuples of varying sizes
macro_rules! impl_from_args_for_tuples {
    // Match for tuples of various sizes
    ($($T:ident),+) => {
        #[allow(non_snake_case)]
        impl<$($T),+> FromArgs for ($($T,)+)
        where
            $($T: 'static + Clone + Send + Sync),+
        {
            fn from_args(args: &[Box<dyn Any + Send + Sync>]) -> Option<Self> {
                if args.len() == count!($($T),+) {
                    Some((
                        $(
                            args[count_index!($T)].downcast_ref::<$T>()?.clone(),
                        )+
                    ))
                } else {
                    None
                }
            }
        }

        #[allow(non_snake_case)]
        impl<$($T),+> IntoArgs for ($($T,)+)
        where
            $($T: 'static + Clone + Send + Sync),+
        {
            fn into_args(self) -> AnyArgs {
                let ($($T,)+) = self;
                vec![$(Box::new($T) as Box<dyn Any + Send + Sync>),+]
            }
        }
    };
}

// Helper macro to count tuple elements
macro_rules! count {
    ($($T:ident),+) => {
        <[()]>::len(&[$(replace_expr!($T ())),+])
    };
}

// Helper macro to generate indices for tuple elements
macro_rules! count_index {
    ($T:ident) => {
        0
    };
    ($T1:ident, $($T:ident),+) => {
        1 + count_index!($($T),+)
    };
}

// Helper macro to replace the expression with a substitute
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

// Implement for tuples of sizes up to 5 (you can extend as needed)
impl_from_args_for_tuples!(T1);
impl_from_args_for_tuples!(T1, T2);
impl_from_args_for_tuples!(T1, T2, T3);
impl_from_args_for_tuples!(T1, T2, T3, T4);
impl_from_args_for_tuples!(T1, T2, T3, T4, T5);

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

    #[test]
    fn test_impl_from_args_for_tuples() {
        let args: Vec<Box<dyn Any + Send + Sync>> = vec![Box::new(1), Box::new(2)];
        let result: Option<(i32, i32)> = FromArgs::from_args(&args);
        assert_eq!(result, Some((1, 2)));
    }
}
