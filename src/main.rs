mod input;
mod spreadsheet;
mod basic;


use std::io::{self, Write};

use input::get_formula;
use spreadsheet::SpreadSheet;

fn main() {
    let stdin = io::stdin();          
    let mut handle = stdin.lock();        
    let mut sheet = SpreadSheet::new(10, 10);

    sheet.print_sheet();
    print!("[0.0] (ok) > ");
    io::stdout().flush().unwrap();
    
    
    loop {
        
    
        
        let state = sheet.call_formula(get_formula(& mut handle));
        sheet.print_sheet();
        match state {
            basic::SpreadSheetError::Valid => print!("[0.0] (ok) > "),
            basic::SpreadSheetError::InvalidInput => print!("[0.0] (invalid input) > "),
            basic::SpreadSheetError::Cycle => print!("[0.0] (cycle) > "),
        }

        io::stdout().flush().unwrap();
        
    
    }
    
}
