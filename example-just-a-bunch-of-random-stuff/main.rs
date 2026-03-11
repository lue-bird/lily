// enabling deref_patterns is sadly required for matching recursive choice types
#![feature(deref_patterns)]
#![allow(incomplete_features)]
mod lily;

fn main() {
    println!(
        "{}",
        lily::str_indent(lily::Str::Slice(
            "test line 1
test line 2
test line 3"
        ))
    );
    println!(
        "{}",
        lily::book()
            .iter()
            .map(lily::Str::as_str)
            .collect::<Vec<&str>>()
            .join("\n")
    );
    let mut lily_state = lily::initial_state();
    for _ in std::iter::repeat_n((), 10) {
        let updated_state_lily = lily::interface(lily_state);
        lily_state = updated_state_lily.new_state;
        println!("{}", updated_state_lily.standard_out_write);
    }
}
