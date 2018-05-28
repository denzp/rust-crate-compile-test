macro_rules! reverse {
    ([$($item:tt),*]) => {
        reverse_impl!([$($item),*], [])
    };
}

macro_rules! reverse_impl {
    ([$head:tt], [$($reversed:tt),*]) => {
        [$head, $($reversed),*]
    };

    ([$head:tt, $($tail:tt),*], []) => {
        reverse_impl!([$($tail),*], [$head])
    };

    ([$head:tt, $($tail:tt),*], [$($reversed:tt),*]) => {
        reverse_impl!([$($tail),*], [$head, $($reversed),*])
    };
}

pub fn reverse() {
    //~ EXPAND t = [4, 3, 2, 6];
    let t = reverse!([6, 2, 3, 4]);
}
