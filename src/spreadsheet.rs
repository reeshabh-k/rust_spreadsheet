use crate::{basic::Cell, basic::Formula, basic::Expression, basic::Value, basic::SpreadSheetError};
use crate::{input::Col};

use std::{cell, collections::HashSet, collections::HashMap};

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

    fn add_cell_list (&self, parent_list: & mut Vec<Cell> , val: Value){
        match val {
            Value::Num(_) => return,
            Value::Ref(c) => parent_list.push(c),
        }
    }

    fn get_pointer (&self, inp_cell: Cell) -> usize {
        inp_cell.row as usize * self.col + inp_cell.col as usize
    }

    fn remove_children (&mut self, inp_cell: Cell) {
        let mut parent_list: Vec<Cell> = vec![];
        let cell_pointer = self.get_pointer(inp_cell);
        let expr = self.spreadsheet[cell_pointer].expr.clone();
        

        match expr {
            Expression::Add(v1, v2) => {
                self.add_cell_list(& mut parent_list, v1);
                self.add_cell_list(& mut parent_list, v2);
            } ,
            Expression::Mul(v1, v2) => {
                self.add_cell_list(& mut parent_list, v1);
                self.add_cell_list(& mut parent_list, v2);
            } ,
            Expression::Div(v1, v2) => {
                self.add_cell_list(& mut parent_list, v1);
                self.add_cell_list(& mut parent_list, v2);
            } ,
            Expression::Sub(v1, v2) => {
                self.add_cell_list(& mut parent_list, v1);
                self.add_cell_list(& mut parent_list, v2);
            } ,
            Expression::Constant(v) => {
                self.add_cell_list(& mut parent_list, v);
            }
            _ => (),
        }

        for i in parent_list.iter().cloned() {
            let parent_pointer = self.get_pointer(i);
            self.spreadsheet[parent_pointer].children.remove(&inp_cell);
        }

    }

    fn add_children (&mut self, inp_cell: Cell, expr: Expression) {
        let mut parent_list: Vec<Cell> = vec![];
        
        

        match expr {
            Expression::Add(v1, v2) => {
                self.add_cell_list(& mut parent_list, v1);
                self.add_cell_list(& mut parent_list, v2);
            } ,
            Expression::Mul(v1, v2) => {
                self.add_cell_list(& mut parent_list, v1);
                self.add_cell_list(& mut parent_list, v2);
            } ,
            Expression::Div(v1, v2) => {
                self.add_cell_list(& mut parent_list, v1);
                self.add_cell_list(& mut parent_list, v2);
            } ,
            Expression::Sub(v1, v2) => {
                self.add_cell_list(& mut parent_list, v1);
                self.add_cell_list(& mut parent_list, v2);
            } ,
            Expression::Constant(v) => {
                self.add_cell_list(& mut parent_list, v);
            }
            _ => (),
        }

        for i in parent_list.iter().cloned() {
            let parent_pointer = self.get_pointer(i);
            self.spreadsheet[parent_pointer].children.insert(inp_cell.clone());
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
        self.remove_children(form.inp_cell.clone());
        self.add_children(form.inp_cell.clone(), form.expression.clone());

        self.update_children(form.inp_cell.clone());

        return SpreadSheetError::Valid;
    }

    fn update_children(&mut self, inp_cell: Cell) {
        let mut cell_counts: HashMap<Cell, u32> = HashMap::new();
        cell_counts.insert(inp_cell.clone(), 0);

        let mut stack: Vec<Cell> = Vec::new();

        stack.push(inp_cell);

        let mut k = 0;

        while stack.is_empty() == false {
            k += 1;
            let top_cell = stack.pop().expect("Stack is empty!");
            
            let cell_pointer = self.get_pointer(top_cell);

            for i in self.spreadsheet[cell_pointer].children.iter() {
                if cell_counts.contains_key(i) == false {
                    stack.push(i.clone());
                }
                cell_counts.insert(i.clone(),   k);   
            }
        }

        let mut sorted_vec: Vec<(Cell, u32)> = cell_counts.into_iter().collect();

        sorted_vec.sort_by(|a, b| a.1.cmp(&b.1));

        for (i, _) in sorted_vec.iter() {
            self.update_cell(i.clone());
        }


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