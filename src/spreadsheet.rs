use crate::{basic::Cell, basic::Formula, basic::Expression, basic::Value, basic::SpreadSheetError, basic::Range};
use crate::input::{Col, get_formula};
use std::io::Cursor;

use std::thread;
use std::time::Duration;

use std::{collections::HashSet, collections::HashMap};

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

    fn recursive_row_split (&self, r : Range) -> Option<(i32, i32, i32, i32)> {
        if r.tl.row == r.br.row {
            let val = self.extract_value_num(Value::Ref(r.tl.clone()))?;
            Some((val, val, val, val*val))
        } else {
            let mid_tl = Cell {
                col: r.tl.col,
                row: (r.tl.row + r.br.row)/2,
            };
            let mid_br = Cell {
                col: r.tl.col,
                row: (r.tl.row + r.br.row)/2 + 1,
            };
            let (v00, v01, v02, v03) = self.recursive_row_split(Range {tl: r.tl, br: mid_tl})?;
            let (v10, v11, v12, v13) = self.recursive_row_split(Range {tl: mid_br, br: r.br})?;
            Some((v00.min(v10), v01.max(v11), v02+v12, v03+v13))
        
        }
    }

    fn recursive_col_split(&self, r: Range) -> Option<(i32, i32, i32, i32)> {
        if r.tl.col == r.br.col {
            self.recursive_row_split(r)
        } else {
            let mid_tl = Cell {
                col: (r.tl.col+r.br.col)/2,
                row: r.br.row,
            };
            let mid_br = Cell {
                col: (r.tl.col+r.br.col)/2 + 1,
                row: r.tl.row,
            };
            let (v00, v01, v02, v03) = self.recursive_col_split(Range {tl: r.tl, br: mid_tl})?;
            let (v10, v11, v12, v13) = (self.recursive_col_split(Range {tl: mid_br, br: r.br}))?;
            Some((v00.min(v10), v01.max(v11), v02+v12, v03+v13))
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
            Expression::Sleep(v) => {
                let sleep_time = self.extract_value_num(v)?;
                thread::sleep(Duration::from_secs(sleep_time as u64));
                Some(sleep_time as i32)

            }

            Expression::Avg(c1, c2) => {
                let (_, _, sum_ele, _) = self.recursive_col_split(Range {tl: c1, br:c2})?;
                Some(sum_ele / ((c2.row as i32  - c1.row as i32+ 1) * (c2.col as i32  - c1.col as i32 + 1)))
            }
            Expression::Max(c1, c2)  => {
                let (_, max_ele, _, _) = self.recursive_col_split(Range {tl: c1, br:c2})?;
                Some(max_ele)
            }
            Expression::Min(c1, c2) => {
                let (min_ele, _, _, _) = self.recursive_col_split(Range {tl: c1, br:c2})?;
                Some(min_ele)
            }
            Expression::Sum(c1, c2) => {
                let (_, _, sum_ele, _) = self.recursive_col_split(Range {tl: c1, br:c2})?;
                Some(sum_ele)
            }
            Expression::Stdev(c1, c2) => {
                let (_, _, sum_ele, square_ele) = self.recursive_col_split(Range {tl: c1, br:c2})?;
                let area = (c2.row as i32  - c1.row as i32+ 1) * (c2.col as i32  - c1.col as i32 + 1);
                let avg = sum_ele/area;
                let sq_avg =( (square_ele/area )as f64).sqrt() as i32;

                Some(sq_avg - avg)
            }

            _ => panic!("Unimplemented Expression Matching in get_expr_res!"),
        }
    }

    fn add_cell_list (&self, parent_list: & mut Vec<Cell> , val: Value){
        match val {
            Value::Num(_) => return,
            Value::Ref(c) => parent_list.push(c),
        }
    }

    fn get_pointer (&self, inp_cell: &Cell) -> usize {
        inp_cell.row as usize * self.col + inp_cell.col as usize
    }

    fn remove_children (&mut self, inp_cell: Cell) {
        let mut parent_list: Vec<Cell> = vec![];
        let cell_pointer = self.get_pointer(&inp_cell);
        let expr = self.spreadsheet[cell_pointer].expr.clone();

       

        match expr {
            Expression::Add(v1, v2)
            | Expression::Mul(v1, v2)
            | Expression::Div(v1, v2)
            | Expression::Sub(v1, v2) => {
                self.add_cell_list(&mut parent_list, v1);
                self.add_cell_list(&mut parent_list, v2);
            }
            
            Expression::Sleep(v) 
            | Expression::Constant(v) => {
                self.add_cell_list(&mut parent_list, v);
            }

            Expression::Avg(c1, c2)
            | Expression::Max(c1, c2)
            | Expression::Min(c1, c2)
            | Expression::Sum(c1, c2)
            | Expression::Stdev(c1, c2) => self.add_range_list(&mut parent_list, c1, c2),
            _ => panic!("Unimplemented add_children!"),
        }

        for i in parent_list.iter().cloned() {
            let parent_pointer = self.get_pointer(&i);
            self.spreadsheet[parent_pointer].children.remove(&inp_cell);
        }

    }

    fn add_range_list (&self, parent_list: & mut Vec<Cell> , cell1: Cell, cell2: Cell) {
        for i in cell1.row..=cell2.row {
            for j in cell1.col..=cell1.col {
                parent_list.push(Cell {row: i, col: j})
            }
        }
    }

    fn add_children (&mut self, inp_cell: Cell, expr: Expression) {
        let mut parent_list: Vec<Cell> = vec![];
        
        match expr {
            Expression::Add(v1, v2)
            | Expression::Mul(v1, v2)
            | Expression::Div(v1, v2)
            | Expression::Sub(v1, v2) => {
                self.add_cell_list(&mut parent_list, v1);
                self.add_cell_list(&mut parent_list, v2);
            }

            Expression::Sleep(v) 
            | Expression::Constant(v) => {
                self.add_cell_list(&mut parent_list, v);
            }

            Expression::Avg(c1, c2)
            | Expression::Max(c1, c2)
            | Expression::Min(c1, c2)
            | Expression::Sum(c1, c2)
            | Expression::Stdev(c1, c2) => self.add_range_list(&mut parent_list, c1, c2),
            
            _ => panic!("Unimplemented add_children!"),
        }

        for i in parent_list.iter().cloned() {
            let parent_pointer = self.get_pointer(&i);
            self.spreadsheet[parent_pointer].children.insert(inp_cell.clone());
        }

    }

    pub fn call_formula (&mut self, form: Option<Formula>) -> SpreadSheetError {
        let form = match form {
            None => return SpreadSheetError::InvalidInput,
            Some(valid_form) => valid_form,
        };
        match form.expression {
            Expression::Quit => return SpreadSheetError::Quit,
            Expression::Disable => return SpreadSheetError::Disable,
            Expression::Enable => return SpreadSheetError::Enable,
            Expression::ScrollDown => {
                self.row_pointer = (self.row_pointer+10).min(self.row - 9);
                return SpreadSheetError::Valid;
            },
            Expression::ScrollUp => {
                self.row_pointer = self.row_pointer.saturating_sub(10).max(1);
                return SpreadSheetError::Valid;
            },
            Expression::ScrollRight => {
                self.col_pointer = (self.col_pointer+10).min(self.col - 9);
                return SpreadSheetError::Valid;
            },
            Expression::ScrollLeft => {
                self.col_pointer =  self.col_pointer.saturating_sub(10).max(1);
                return SpreadSheetError::Valid;
            },
            Expression::ScrollTo(c) => {
                self.col_pointer = c.col as usize;
                self.row_pointer = c.row as usize;
                return SpreadSheetError::Valid;
            },
            Expression::Avg(c1, c2)
            | Expression::Max(c1, c2)
            | Expression::Min(c1, c2)
            | Expression::Sum(c1, c2)
            | Expression::Stdev(c1, c2) => {
                if c1.row > c2.row || c1.col > c2.col {
                    return SpreadSheetError::InvalidInput;
                }
            }
            _ => ()
        }
        match self.check_cycle(form.clone()) {
            true => return SpreadSheetError::Cycle,
            false => (),
        }

        
        self.remove_children(form.inp_cell.clone());

        let cell_pointer = self.col*form.inp_cell.row as usize + form.inp_cell.col as usize;
        self.spreadsheet[cell_pointer].expr = form.expression.clone();

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
            
            let cell_pointer = self.get_pointer(&top_cell);

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

    fn belongs_to_expression(&self, expr : &Expression, c: Cell) -> bool {
        let val = Value::Ref(c.clone());

        match expr {
            | Expression::Add(v1, v2)
            | Expression::Div(v1, v2)
            | Expression::Mul(v1, v2)
            | Expression::Sub(v1, v2) => *v1 == val || *v2 == val,


            Expression::Sleep(v)
            | Expression::Constant(v) => *v == val,

            Expression::Avg(c1, c2)
            | Expression::Max(c1, c2)
            | Expression::Min(c1, c2)
            | Expression::Sum(c1, c2)
            | Expression::Stdev(c1, c2) => {
                c.row >= c1.row && c.col >= c1.col && c.row <= c2.row && c.col <= c2.col
            },


            _ => panic!("Unimplemented belongs_to_expression!"),
        }
    }

    fn check_cycle (&self, form: Formula) -> bool {
        let inp_cell = form.inp_cell.clone();
        let expr = form.expression;

        if self.belongs_to_expression(&expr, inp_cell.clone()) {
            return true;
        }

        let mut visited: HashSet<Cell> = HashSet::new();
        visited.insert(inp_cell.clone());

        let mut stack: Vec<Cell> = Vec::new();
        stack.push(inp_cell);

        while stack.is_empty() == false {
            let top_cell = stack.pop().expect("Stack is empty!");
            
            let cell_pointer = self.get_pointer(&top_cell);

            for i in self.spreadsheet[cell_pointer].children.iter() {
                if self.belongs_to_expression(&expr, i.clone()) {
                    return true;
                }
                if visited.contains(i) == false {
                    stack.push(i.clone());
                    visited.insert(i.clone());   
                }    
            }
        }
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
                if self.spreadsheet[cell_pointer+j as usize].valid == 1 {
                    print!("{:<10}", "err");
                }
                else {
                    print!("{:<10}", self.spreadsheet[cell_pointer + j as usize].val);
                }
            }
            println!();
        }
        
    }
}

#[cfg(test)]
mod col_tests {
    use super::*;

    fn test_range (v: Vec<&str>, out: (i32, i32, i32, i32)) {
        let mut spreadsheet = SpreadSheet::new(10,10);

        for i in v.iter() {
            let mut inp = Cursor::new(*i);
            let parsed_input = get_formula(&mut inp);
            spreadsheet.call_formula(parsed_input);
        }

        assert_eq!(spreadsheet.recursive_col_split(Range {tl: Cell {col: 1, row: 1}, br: Cell {col: 10, row: 10}}).expect(""), out);
    }

    #[test]
    fn range_1 () {
        test_range(vec!["A1=1"], (0,1,1,1));
        test_range(vec!["A1=1", "B1=1"], (0,1,2,2));
        test_range(vec!["A1=1", "B1=1", "C1=1", "D1=1", "E1=1"], (0,1,5,5));
        test_range(vec!["A1=-1", "A2 = 3"],     (-1, 3, 2, 10));
    }

}