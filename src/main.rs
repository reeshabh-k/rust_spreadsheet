//! A simple spreadsheet system where users can input formulas and track dependencies.
//! 
//! # Modules
//! - `basic`: Contains common error handling and state definitions for the spreadsheet.
//! - `input`: Handles input for formulas from stdin.
//! - `spreadsheet`: Manages the creation, modification, and printing of the spreadsheet.
//!
//! The `main` function initializes the spreadsheet, handles user input, and manages the 
//! processing of formulas within the spreadsheet.
//!
//! ## Arguments
//! - If no command-line arguments are provided, the program initializes with a default size.
//! - If two command-line arguments are provided, they are used to define the number of rows and columns in the spreadsheet.
//! 
//! # Example
//! ```sh
//! $ cargo run 100 100
//! ```
//! Will create a spreadsheet with 100 rows and 100 columns.
//!
//! ## Program Flow
//! The program continuously accepts formulas as input, processes them, and updates the sheet.
//! It will print the state of the spreadsheet after each formula is processed unless disabled.
//! The user can also disable or enable sheet printing using specific commands in the input formula.
 
mod basic;
mod input;
mod spreadsheet;

use std::env;
use std::io::{self, Write};
use std::time::Instant;

use input::get_formula;
use spreadsheet::SpreadSheet;


/// Main entry point for the spreadsheet program.
///
/// Initializes the spreadsheet based on command-line arguments (if provided) and enters
/// a loop to accept and process formulas from the user. It tracks the time taken to process
/// each formula and updates the spreadsheet accordingly.
///
/// # Arguments
/// - If no arguments are provided, the default row and column sizes are 999 and 18278 respectively.
/// - If two arguments are provided, they are interpreted as the row and column sizes for the spreadsheet.
///
/// # Flow
/// The program continuously asks for formulas and processes them, printing the current state
/// of the spreadsheet. The user can disable or enable sheet printing during execution.
#[cfg(not(tarpaulin))]
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
