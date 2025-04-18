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
    let mut sheet = SpreadSheet::new(31, 52);

    sheet.print_sheet();
    print!("[0.0] (ok) > ");
    io::stdout().flush().unwrap();

    let mut print_sheet = true;
    
    
    loop {
        
    
        let form = get_formula(& mut handle);

        let start = Instant::now();
        let state = sheet.call_formula(form);
        let duration = start.elapsed();

        let mut dashboard = String::new();

        

        match state {
            basic::SpreadSheetError::Valid => dashboard = format!("[{:.5}] (ok) > ", duration.as_secs_f32()),
            basic::SpreadSheetError::InvalidInput => dashboard = format!("[{:.5}] (invalid input) > ", duration.as_secs_f32()),
            basic::SpreadSheetError::Cycle => dashboard = format!("[{:.5}] (cycle) > ", duration.as_secs_f32()),
            basic::SpreadSheetError::Quit => break,
            basic::SpreadSheetError::Disable => {
                dashboard = format!("[{:.5}] (ok) > ", duration.as_secs_f32());
                print_sheet = false;
            },
            basic::SpreadSheetError::Enable => {
                dashboard = format!("[{:.5}] (ok) > ", duration.as_secs_f32());
                print_sheet = true;
            }
        }

        if print_sheet == true{
            sheet.print_sheet();
        }
        print!("{}", dashboard);

        io::stdout().flush().unwrap();
        
    
    }

    println!("************");
    
}
