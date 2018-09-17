#![feature(crate_visibility_modifier)]

use std::process::exit;

//  main.rs is a different crate than lib.rs, so the `crate` keyword in lib
//  is the name `ysh` here.
use ysh::st::State;

use crossterm::Screen;

fn main() {
    // Put it in raw mode
    let mut screen = Screen::new(true);
    State::default()
        .init(&mut screen)
        .and_then(move |mut state| state.run(screen))
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            exit(1);
        });
}
