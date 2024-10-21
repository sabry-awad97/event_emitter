use std::any::Any;

// Trait for converting into boxed Any arguments
pub trait IntoArgs {
    fn into_args(self) -> Vec<Box<dyn Any + Send + Sync>>;
}

// Macro to implement IntoArgs for tuples
macro_rules! impl_into_args {
    () => {
        impl IntoArgs for () {
            fn into_args(self) -> Vec<Box<dyn Any + Send + Sync>> {
                Vec::new()
            }
        }
    };
    ($($T:ident),+) => {
        impl<$($T),+> IntoArgs for ($($T,)+)
        where
            $($T: 'static + Send + Sync,)+
        {
            #[allow(non_snake_case)]
            fn into_args(self) -> Vec<Box<dyn Any + Send + Sync>> {
                let ($($T,)+) = self;
                vec![$(Box::new($T),)+]
            }
        }
    };
}

// Generate implementations for tuples of 0 to 5 elements
impl_into_args!();
impl_into_args!(T0);
impl_into_args!(T0, T1);
impl_into_args!(T0, T1, T2);
impl_into_args!(T0, T1, T2, T3);
impl_into_args!(T0, T1, T2, T3, T4);

// Trait for converting from boxed Any arguments
pub trait FromArgs: Sized {
    fn from_args(args: &[Box<dyn Any + Send + Sync>]) -> Option<Self>;
}

// Helper macro to count tuple elements
macro_rules! count {
    () => { 0 };
    ($T:ident $(,$rest:ident)*) => { 1 + count!($($rest),*) };
}

// Macro to implement FromArgs for tuples
macro_rules! impl_from_args {
    () => {
        impl FromArgs for () {
            fn from_args(args: &[Box<dyn Any + Send + Sync>]) -> Option<Self> {
                if args.is_empty() {
                    Some(())
                } else {
                    None
                }
            }
        }
    };
    ($($T:ident),+) => {
        #[allow(unused_assignments)]
        impl<$($T),+> FromArgs for ($($T,)+)
        where
            $($T: 'static + Clone + Send + Sync,)+
        {
            fn from_args(args: &[Box<dyn Any + Send + Sync>]) -> Option<Self> {
                if args.len() == count!($($T),+) {
                    let mut index = 0;
                    Some((
                        $(
                            {
                                let arg = args[index].downcast_ref::<$T>()?.clone();
                                index += 1;
                                arg
                            },
                        )+
                    ))
                } else {
                    None
                }
            }
        }
    };
}

// Generate implementations for tuples of 0 to 5 elements
impl_from_args!();
impl_from_args!(T0);
impl_from_args!(T0, T1);
impl_from_args!(T0, T1, T2);
impl_from_args!(T0, T1, T2, T3);
impl_from_args!(T0, T1, T2, T3, T4);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impl_from_args_for_tuples() {
        let args: Vec<Box<dyn Any + Send + Sync>> = vec![Box::new(1), Box::new(2)];
        let result: Option<(i32, i32)> = FromArgs::from_args(&args);
        assert_eq!(result, Some((1, 2)));
    }

    #[test]
    fn test_count_macro() {
        assert_eq!(count!(), 0);
        assert_eq!(count!(A), 1);
        assert_eq!(count!(A, B), 2);
        assert_eq!(count!(A, B, C), 3);
        assert_eq!(count!(A, B, C, D, E), 5);
    }

    #[test]
    fn test_impl_into_args_for_tuples() {
        let tuple = (1, "hello", std::f64::consts::PI);
        let args = tuple.into_args();
        assert_eq!(args.len(), 3);
        assert!(args[0].downcast_ref::<i32>().is_some());
        assert!(args[1].downcast_ref::<&str>().is_some());
        assert!(args[2].downcast_ref::<f64>().is_some());
    }
}
