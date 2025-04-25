mod basic;
mod input;
mod spreadsheet;

use std::env;
use std::io::{self, Write};
use std::time::Instant;

use input::get_formula;
use spreadsheet::SpreadSheet;

fn main() {
    let args: Vec<String> = env::args().collect();

    let row;
    let col;

    if args.len() != 3 {
        row = 999;
        col = 18278;
    } else {
        row = args[1].parse::<usize>().expect("Incorrect Argument");
        col = args[2].parse::<usize>().expect("Incorrect Argument");
    }

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut sheet = SpreadSheet::new(row, col);

    sheet.print_sheet();
    print!("[0.0] (ok) > ");
    io::stdout().flush().unwrap();

    let mut print_sheet = true;

    loop {
        let form = get_formula(&mut handle);

        let start = Instant::now();
        let state = sheet.call_formula(form);
        let duration = start.elapsed();

        let dashboard;

        match state {
            basic::SpreadSheetError::Valid => {
                dashboard = format!("[{:.10}] (ok) > ", duration.as_secs_f32())
            }
            basic::SpreadSheetError::InvalidInput | basic::SpreadSheetError::Cycle => {
                dashboard = format!("[{:.10}] (err) > ", duration.as_secs_f32())
            }
            basic::SpreadSheetError::Quit => break,
            basic::SpreadSheetError::Disable => {
                dashboard = format!("[{:.10}] (ok) > ", duration.as_secs_f32());
                print_sheet = false;
            }
            basic::SpreadSheetError::Enable => {
                dashboard = format!("[{:.10}] (ok) > ", duration.as_secs_f32());
                print_sheet = true;
            }
        }

        if print_sheet {
            sheet.print_sheet();
        }
        print!("{dashboard}");

        io::stdout().flush().unwrap();
    }

    println!("************");
}
