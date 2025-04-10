mod input;

use std::io;

fn main() {
    let stdin = io::stdin();          
    let mut handle = stdin.lock();        
    input::get_formula(& mut handle).expect("Incorrect Input");
    input::get_formula(& mut handle).expect("Incorrect Input");
}
