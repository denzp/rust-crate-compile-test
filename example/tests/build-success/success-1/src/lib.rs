mod mod1 {
    use mod2::func2;

    fn func1() -> u32 {
        func2()
    }
}

mod mod2 {
    pub fn func2() -> u32 {
        0
    }
}
