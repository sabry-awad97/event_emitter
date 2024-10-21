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
