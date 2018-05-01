mod mod1 {
    use mod2::func3;

    fn func1() -> u32 {
        func3()
    }
}

mod mod2 {
    fn func2() -> NonExistingType {
        0
    }
}
