mod mod_1;
mod mod_2;

fn test() -> u32 {
    //~^ ERROR E0308
    mod_1::func1();
    //~^ ERROR function `func1` is private
}
