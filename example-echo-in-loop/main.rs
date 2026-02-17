// enabling deref_patterns is sadly required for matching recursive choice types
#![feature(deref_patterns)]
#![allow(incomplete_features)]

mod still;

/// you'll most likely want to introduce an alias for this on the still side instead
type StillState = still::Str;

fn main() {
    let mut still_state: still::Opt<StillState> = still::Opt::Absent;
    'main_loop: loop {
        let interface = still::interface(still_state);
        let maybe_new_state: Option<StillState> = handle_io(&interface);
        match maybe_new_state {
            None => {
                break 'main_loop;
            }
            Some(new_still_state) => {
                still_state = still::Opt::Present(new_still_state);
            }
        }
    }
}
/// returns a new state
fn handle_io(interface: &still::Io<StillState>) -> Option<StillState> {
    match interface {
        still::Io::Standard_out_write(to_write) => {
            print!("{}", to_write);
            let _ = std::io::Write::flush(&mut std::io::stdout());
            None
        }
        still::Io::Batch(ios) => {
            for io in ios.iter() {
                if let Some(new_state) = handle_io(io) {
                    return Some(new_state);
                }
            }
            None
        }
        still::Io::Standard_in_read_line(on_read_line) => {
            let mut read_line: String = String::new();
            let _ = std::io::stdin().read_line(&mut read_line);
            Some(on_read_line(still::Str::from_string(read_line)))
        }
    }
}
