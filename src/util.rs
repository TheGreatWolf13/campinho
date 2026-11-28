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

#[macro_export]
macro_rules! seq_literal {
    ($n: ident in ($($l:literal),+) $b:tt) => {
        super_seq_macro::seq!($n in (0..[$($l),+].len()).collect().map(|i| [$($l),+][i]) {
            paste::paste! $b
        });
    };
}