mod nested_mod_1;

macro_rules! custom_fn {
    ($name:ident() $body:block) => {
        pub fn $name(_arg: u32) -> u32 $body
    };
}

//~ EXPAND pub fn other_fn(_arg: u32) -> u32 {
//~ EXPAND     0
//~ EXPAND }

custom_fn!(other_fn() {
    0
});


mod inner_mod {
    custom_fn!(inner_fn() {
        1
    });
}