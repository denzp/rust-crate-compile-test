use mod_2::func3; //~  WARNING another warning
                  //~^ ERROR   E0432
                  //~| ERROR   unresolved import `mod_2::func3`

fn func1() -> u32 {
    func3()
}
