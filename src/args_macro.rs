use std::any::Any;

// Trait for converting into boxed Any arguments
pub trait IntoArgs {
    fn into_args(self) -> Vec<Box<dyn Any + Send + Sync>>;
}

// Implement IntoArgs for tuples from 1 to 5 elements
impl<T1> IntoArgs for (T1,)
where
    T1: 'static + Send + Sync,
{
    fn into_args(self) -> Vec<Box<dyn Any + Send + Sync>> {
        vec![Box::new(self.0)]
    }
}

impl<T1, T2> IntoArgs for (T1, T2)
where
    T1: 'static + Send + Sync,
    T2: 'static + Send + Sync,
{
    fn into_args(self) -> Vec<Box<dyn Any + Send + Sync>> {
        vec![Box::new(self.0), Box::new(self.1)]
    }
}

impl<T1, T2, T3> IntoArgs for (T1, T2, T3)
where
    T1: 'static + Send + Sync,
    T2: 'static + Send + Sync,
    T3: 'static + Send + Sync,
{
    fn into_args(self) -> Vec<Box<dyn Any + Send + Sync>> {
        vec![Box::new(self.0), Box::new(self.1), Box::new(self.2)]
    }
}

impl<T1, T2, T3, T4> IntoArgs for (T1, T2, T3, T4)
where
    T1: 'static + Send + Sync,
    T2: 'static + Send + Sync,
    T3: 'static + Send + Sync,
    T4: 'static + Send + Sync,
{
    fn into_args(self) -> Vec<Box<dyn Any + Send + Sync>> {
        vec![
            Box::new(self.0),
            Box::new(self.1),
            Box::new(self.2),
            Box::new(self.3),
        ]
    }
}

impl<T1, T2, T3, T4, T5> IntoArgs for (T1, T2, T3, T4, T5)
where
    T1: 'static + Send + Sync,
    T2: 'static + Send + Sync,
    T3: 'static + Send + Sync,
    T4: 'static + Send + Sync,
    T5: 'static + Send + Sync,
{
    fn into_args(self) -> Vec<Box<dyn Any + Send + Sync>> {
        vec![
            Box::new(self.0),
            Box::new(self.1),
            Box::new(self.2),
            Box::new(self.3),
            Box::new(self.4),
        ]
    }
}

// Trait for converting from boxed Any arguments
pub trait FromArgs: Sized {
    fn from_args(args: &[Box<dyn Any + Send + Sync>]) -> Option<Self>;
}

// Helper macro to count tuple elements
macro_rules! count {
    () => { 0 };
    ($T:ident $(,$rest:ident)*) => { 1 + count!($($rest),*) };
}

// Helper macro to generate indices for tuple elements
macro_rules! count_index {
    ($T:ident) => { 0 };
    ($T:ident, $($rest:ident),+) => { 1 + count_index!($($rest),+) };
}

// Replace the previous impl_from_args macro with this corrected version:

macro_rules! impl_from_args {
    ($($T:ident),+) => {
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

// Generate implementations for tuples of 1 to 5 elements
impl_from_args!(T1);
impl_from_args!(T1, T2);
impl_from_args!(T1, T2, T3);
impl_from_args!(T1, T2, T3, T4);
impl_from_args!(T1, T2, T3, T4, T5);

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
    fn test_count_index_macro() {
        assert_eq!(count_index!(A), 0);
        assert_eq!(count_index!(A, B), 1);
        assert_eq!(count_index!(A, B, C), 2);
        assert_eq!(count_index!(A, B, C, D), 3);
        assert_eq!(count_index!(A, B, C, D, E), 4);
    }
}
