mod input;
mod spreadsheet;
mod basic;


use std::io::{self, Write};
use std::time::Instant;

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
        
    
        let form = get_formula(& mut handle);

        let start = Instant::now();
        let state = sheet.call_formula(form);
        let duration = start.elapsed();
        sheet.print_sheet();

        match state {
            basic::SpreadSheetError::Valid => print!("[{:.5}] (ok) > ", duration.as_secs_f32()),
            basic::SpreadSheetError::InvalidInput => print!("[{:.5}] (invalid input) > ", duration.as_secs_f32()),
            basic::SpreadSheetError::Cycle => print!("[{:.5}] (cycle) > ", duration.as_secs_f32()),
            basic::SpreadSheetError::Quit => break,
        }

        io::stdout().flush().unwrap();
        
    
    }

    println!("************");
    
}
