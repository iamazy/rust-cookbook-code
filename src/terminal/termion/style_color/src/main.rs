extern crate termion;

use std::io;
use termion::{color, style};

fn main() {
    /// sytles and colors
    println!("{}Red", color::Fg(color::Red));
    println!("{}Blue", color::Fg(color::Blue));
    println!("{}Blue 'n' Bold{}", style::Bold, style::Reset);
    println!("{}Just plain italic", style::Italic);

    // moveing the cursor
    println!("{}{}Stuff",termion::clear::All, termion::cursor::Goto(1,10));
}
