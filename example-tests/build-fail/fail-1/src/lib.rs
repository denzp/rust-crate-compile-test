mod mod1 {
    use mod2::func3; //~  WARNING another warning
                     //~^ ERROR   E0432
                     //~| ERROR   unresolved import `mod2::func3`

    fn func1() -> u32 {
        func3()
    }
}

mod mod2 {
    fn func2() -> NonExistingType {
        0
    }
    //~^^^ ERROR E0433
}
// ~  NOTE With extra space
// ~| HELP For previous line
