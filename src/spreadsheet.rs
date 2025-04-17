use crate::{basic::Cell, basic::Formula, basic::Expression, basic::Value, basic::SpreadSheetError};
use crate::{input::Col};

use std::{cell, collections::HashSet};

#[derive(Clone, Debug)]
struct CellData {
    val: i32,
    expr: Expression,
    valid: u8,
    children: HashSet<Cell>
}

pub struct SpreadSheet {
    row_pointer: usize,
    col_pointer: usize,
    row: usize,
    col: usize,
    spreadsheet: Vec<CellData>,
}

impl SpreadSheet {
    pub fn new (row: usize, col: usize) -> SpreadSheet {
        let default_cell = CellData {
            val: 0,
            expr: Expression::Constant(Value::Num(0)),
            valid: 0 as u8,
            children: HashSet::new(),
        };
        SpreadSheet { row_pointer: 1, col_pointer: 1, row, col, spreadsheet: vec![default_cell; (row+1)*(1+col)] }
    }

    fn update_cell (&mut self, cell: Cell) {
        let row = cell.row as usize;
        let col = cell.col as usize;

        let cell_loc = self.col*row + col;
        let cell_data = &self.spreadsheet[cell_loc];
    
        let expr = cell_data.expr.clone();

        let eval_expr = self.get_expr_res(expr);

        match eval_expr {
            None => self.spreadsheet[cell_loc].valid = 1,
            Some(i) => {
                self.spreadsheet[cell_loc].valid = 0;
                self.spreadsheet[cell_loc].val = i
            }
        }
    }

    fn extract_value_num (&self, val: Value)-> Option<i32> {
        match val {
            Value::Num(i) => Some(i),
            Value::Ref(cell) => {
                let cell_point = self.col * (cell.row as usize)  + (cell.col as usize);
                if self.spreadsheet[cell_point].valid == 1 {
                    None
                } else {
                    Some(self.spreadsheet[cell_point].val.clone())
                }
            }
        }
    }


    fn get_expr_res (&self, expr: Expression) -> Option<i32> {
        match expr {
            Expression::Add(v1, v2) => Some(self.extract_value_num(v1)? + self.extract_value_num(v2)?),
            Expression::Mul(v1, v2) => Some(self.extract_value_num(v1)? * self.extract_value_num(v2)?),
            Expression::Div(v1, v2) => {
                let denom = self.extract_value_num(v2)?;
                if denom == 0 {
                    None
                } else {
                    Some(self.extract_value_num(v1)? / denom) 
                }
            },
            Expression::Sub(v1, v2) => Some(self.extract_value_num(v1)? - self.extract_value_num(v2)?),
            Expression::Constant(v) => self.extract_value_num(v),
            _ => panic!("Unimplemented Expression Matching in get_expr_res!"),
        }
    }

    pub fn call_formula (&mut self, form: Option<Formula>) -> SpreadSheetError {
        let form = match form {
            None => return SpreadSheetError::InvalidInput,
            Some(valid_form) => valid_form,
        };
        match self.check_cycle(form.clone()) {
            true => return SpreadSheetError::Cycle,
            false => (),
        }

        let cell_pointer = self.col*form.inp_cell.row as usize + form.inp_cell.col as usize;
        self.spreadsheet[cell_pointer].expr = form.expression.clone();
        self.update_cell(form.inp_cell);

        return SpreadSheetError::Valid;
    }

    fn check_cycle (&self, form: Formula) -> bool {
        false
    }

    pub fn print_sheet (&self) {
        let width = 10.min(self.col as u32 - self.col_pointer as u32 + 1);
        let length = 10.min(self.row as u32 - self.row_pointer as u32 + 1);
        print!("          ");
        for i in 0..width {
            let col = Col::from_num(self.col_pointer as u32 + i).expect("Col Pointer is at Invalid Location");
            print!("{:<10}", col.as_str());
        }
        println!();
        for i in 0..length {
            print!("{:<10}", self.row_pointer + i as usize);
            let cell_pointer = self.col * (i as usize + self.row_pointer) + self.col_pointer;
            for j in 0..width {
                print!("{:<10}", self.spreadsheet[cell_pointer + j as usize].val);
            }
            println!();
        }
        
    }
}