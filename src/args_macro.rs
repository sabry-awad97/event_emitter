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

// Implement FromArgs for tuples from 1 to 5 elements
impl<T1> FromArgs for (T1,)
where
    T1: 'static + Clone + Send + Sync,
{
    fn from_args(args: &[Box<dyn Any + Send + Sync>]) -> Option<Self> {
        if args.len() == 1 {
            let arg1 = args[0].downcast_ref::<T1>()?;
            Some((arg1.clone(),))
        } else {
            None
        }
    }
}

impl<T1, T2> FromArgs for (T1, T2)
where
    T1: 'static + Clone + Send + Sync,
    T2: 'static + Clone + Send + Sync,
{
    fn from_args(args: &[Box<dyn Any + Send + Sync>]) -> Option<Self> {
        if args.len() == 2 {
            let arg1 = args[0].downcast_ref::<T1>()?;
            let arg2 = args[1].downcast_ref::<T2>()?;
            Some((arg1.clone(), arg2.clone()))
        } else {
            None
        }
    }
}

impl<T1, T2, T3> FromArgs for (T1, T2, T3)
where
    T1: 'static + Clone + Send + Sync,
    T2: 'static + Clone + Send + Sync,
    T3: 'static + Clone + Send + Sync,
{
    fn from_args(args: &[Box<dyn Any + Send + Sync>]) -> Option<Self> {
        if args.len() == 3 {
            let arg1 = args[0].downcast_ref::<T1>()?;
            let arg2 = args[1].downcast_ref::<T2>()?;
            let arg3 = args[2].downcast_ref::<T3>()?;
            Some((arg1.clone(), arg2.clone(), arg3.clone()))
        } else {
            None
        }
    }
}

impl<T1, T2, T3, T4> FromArgs for (T1, T2, T3, T4)
where
    T1: 'static + Clone + Send + Sync,
    T2: 'static + Clone + Send + Sync,
    T3: 'static + Clone + Send + Sync,
    T4: 'static + Clone + Send + Sync,
{
    fn from_args(args: &[Box<dyn Any + Send + Sync>]) -> Option<Self> {
        if args.len() == 4 {
            let arg1 = args[0].downcast_ref::<T1>()?;
            let arg2 = args[1].downcast_ref::<T2>()?;
            let arg3 = args[2].downcast_ref::<T3>()?;
            let arg4 = args[3].downcast_ref::<T4>()?;
            Some((arg1.clone(), arg2.clone(), arg3.clone(), arg4.clone()))
        } else {
            None
        }
    }
}

impl<T1, T2, T3, T4, T5> FromArgs for (T1, T2, T3, T4, T5)
where
    T1: 'static + Clone + Send + Sync,
    T2: 'static + Clone + Send + Sync,
    T3: 'static + Clone + Send + Sync,
    T4: 'static + Clone + Send + Sync,
    T5: 'static + Clone + Send + Sync,
{
    fn from_args(args: &[Box<dyn Any + Send + Sync>]) -> Option<Self> {
        if args.len() == 5 {
            let arg1 = args[0].downcast_ref::<T1>()?;
            let arg2 = args[1].downcast_ref::<T2>()?;
            let arg3 = args[2].downcast_ref::<T3>()?;
            let arg4 = args[3].downcast_ref::<T4>()?;
            let arg5 = args[4].downcast_ref::<T5>()?;
            Some((
                arg1.clone(),
                arg2.clone(),
                arg3.clone(),
                arg4.clone(),
                arg5.clone(),
            ))
        } else {
            None
        }
    }
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
