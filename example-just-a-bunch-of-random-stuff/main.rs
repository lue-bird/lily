mod still;

fn main() {
    let mut allocator: bumpalo::Bump = bumpalo::Bump::new();
    let mut still_state = still::initial_state();
    println!(
        "{}",
        still::book()
            .iter()
            .map(still::Str::as_str)
            .collect::<Vec<&str>>()
            .join("\n")
    );
    for _ in std::iter::repeat_n((), 10) {
        let updated_state_still =
            still::interface(still::OwnedToStill::into_still(still_state, &allocator));
        still_state = still::StillIntoOwned::into_owned(updated_state_still.new_state);
        println!("{}", updated_state_still.standard_out_write);
        allocator.reset();
    }
}
impl still::Alloc for bumpalo::Bump {
    fn alloc<A>(&self, value: A) -> &A {
        self.alloc(value)
    }
}
