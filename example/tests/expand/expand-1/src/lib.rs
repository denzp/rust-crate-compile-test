mod mod_1;
mod mod_2;

macro_rules! sum {
    ($lhs:expr, $rhs:expr) => {
        $lhs + $rhs
    };
}

fn some_fn() {
    //~ EXPAND x = 2;
    //~ EXPAND x = 1 + 1;
    let x = sum!(1, 1);

    let mut y;

    //~ EXPAND y = 1 + 1;
    //~ EXPAND y = 2 + 2;
    //~ EXPAND y = 4 + 4;

    y = sum!(1, 1);
    y = sum!(1, 2);
    y = sum!(3, 3);
    y = sum!(4, 4);
}
