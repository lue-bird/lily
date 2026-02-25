// enabling deref_patterns is sadly required for matching recursive choice types
#![feature(deref_patterns)]
#![allow(incomplete_features)]
mod still;

fn main() {
    println!(
        "{}",
        still::book()
            .iter()
            .map(still::Str::as_str)
            .collect::<Vec<&str>>()
            .join("\n")
    );
    let mut still_state = still::initial_state();
    for _ in std::iter::repeat_n((), 10) {
        let updated_state_still = still::interface(still_state);
        still_state = updated_state_still.new_state;
        println!("{}", updated_state_still.standard_out_write);
    }
}
