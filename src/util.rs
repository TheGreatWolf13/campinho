#[macro_export]
macro_rules! if_else {
    ($condition:expr => $if_true:expr ; $if_false:expr) => {
        if $condition {
            $if_true
        } //
        else {
            $if_false
        }
    };
}