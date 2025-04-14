mod input;

use std::io;

fn main() {
    let stdin = io::stdin();          
    let mut handle = stdin.lock();        
    input::get_formula(& mut handle);
    input::get_formula(& mut handle);
}
