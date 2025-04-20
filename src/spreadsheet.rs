use crate::{basic::Cell, basic::Formula, basic::Expression, basic::Value, basic::SpreadSheetError, basic::Range};
use crate::input::{Col, get_formula};
use std::io::Cursor;

use std::thread;
use std::time::Duration;

use std::{collections::HashSet, collections::HashMap};

#[derive(Clone, Debug)]
struct CellData {
    children: HashSet<Cell>
}

pub struct SpreadSheet {
    row_pointer: usize,
    col_pointer: usize,
    row: usize,
    col: usize,
    spreadsheet: Vec<CellData>,
    val: Vec<i32>,
    valid: Vec<u8>,
    exprs: HashMap<Cell, Expression>,
}

impl SpreadSheet {
    pub fn new (row: usize, col: usize) -> SpreadSheet {
        let default_cell = CellData {
            
            children: HashSet::new(),
        };
        SpreadSheet { row_pointer: 1, col_pointer: 1, row, col, spreadsheet: vec![default_cell; (row+1)*(1+col)] , val: vec![0; (row+1)*(col+1)], valid: vec![0; (row+1)*(col+1)], exprs: HashMap::new()}
    }

    fn update_cell (&mut self, cell: Cell) {
        let row = cell.row as usize;
        let col = cell.col as usize;

        let cell_loc = self.col*row + col;
        let cell_data = &self.spreadsheet[cell_loc];

        let expr;
    
        if self.exprs.contains_key(&cell) {
            expr = self.exprs.get(&cell).expect("Weird!");
        } else {
            return;
        }

        let eval_expr = self.get_expr_res(expr.clone());

        match eval_expr {
            None => self.valid[cell_loc] = 1,
            Some(i) => {
                self.valid[cell_loc] = 0;
                self.val[cell_loc] = i
            }
        }
    }

    fn extract_value_num (&self, val: Value)-> Option<i32> {
        match val {
            Value::Num(i) => Some(i),
            Value::Ref(cell) => {
                let cell_point = self.col * (cell.row as usize)  + (cell.col as usize);
                if self.valid[cell_point] == 1 {
                    None
                } else {
                    Some(self.val[cell_point].clone())
                }
            }
        }
    }

    fn recursive_row_split (&self, r : Range) -> Option<(i32, i32, i32, i32)> {
        if r.tl.row == r.br.row {
            // let val = self.extract_value_num(Value::Ref(r.tl.clone()))?;
            // Some((val, val, val, val*val))
            self.recursive_col_split(r)
        } else {
            let mid_tl = Cell {
                col: r.br.col,
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
            
            let val = self.extract_value_num(Value::Ref(r.tl.clone()))?;
            Some((val, val, val, val*val))
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

    fn get_sum(&self, c1: Cell, c2: Cell) -> Option<i32> {
        let mut sum = 0;
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                // if self.spreadsheet[self.get_pointer(&Cell {row : i, col : j})].valid == 1 {
                //     return None;
                // }
                sum += self.val[self.get_pointer(&Cell {row : i, col : j})];
            }
        }
        Some(sum)
    }

    fn get_avg(&self, c1: Cell, c2: Cell) ->Option<i32>{
        let mut sum = 0;
        let mut cnt = 0;
        // sum = self.spreadsheet[self.get_pointer(&Cell {row : c1.row, col : c1.col})].val;
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell {row : i, col : j})] == 1 {
                    return None;
                }
                sum += self.val[self.get_pointer(&Cell {row : i, col : j})];
                cnt += 1;
            }
        }
        let mean = sum / cnt;
        Some(mean)
    }

    fn get_max(&self, c1: Cell, c2: Cell) -> Option<i32> {
        let mut max;
        // let mut min;
        max = self.val[self.get_pointer(&Cell {row : c1.row, col : c1.col})];
        // min = self.spreadsheet[self.get_pointer(&Cell {row : c1.row, col : c1.col})].val;
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell {row : i, col : j})] == 1 {
                    return None;
                }
                let val = self.val[self.get_pointer(&Cell {row : i, col : j})];
                if val > max {
                    max = val;
                }
                // if val < min {
                //     min = val;
                // }
            }
        }
        Some(max)
    }

    fn get_min(&self, c1: Cell, c2: Cell) ->Option<i32> {
        // let mut max;
        let mut min;
        // max = self.spreadsheet[self.get_pointer(&Cell {row : c1.row, col : c1.col})].val;
        min = self.val[self.get_pointer(&Cell {row : c1.row, col : c1.col})];
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell {row : i, col : j})] == 1 {
                    return None;
                }
                let val = self.val[self.get_pointer(&Cell {row : i, col : j})];
                // if val > max {
                //     max = val;
                // }
                if val < min {
                    min = val;
                }
            }
        }
        Some(min)
    }

    fn get_stddev(&self, c1: Cell, c2: Cell) -> Option<i32> {
        let mut mean = 0;
        let mut variance = 0.0;
        let mut cnt = 0;
        let mut temp ;
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell {row : i, col : j})] == 1 {
                    return None;
                }
                mean += self.val[self.get_pointer(&Cell {row : i, col : j})];
                cnt += 1;
            }
        }
        mean /= cnt;

        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                temp = self.val[self.get_pointer(&Cell {row : i, col : j})];
                variance += ((temp - mean) * (temp - mean)) as f64;
            }
        }
        variance /= cnt as f64;
        let stddev = variance.sqrt();
        Some(stddev.round() as i32)
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
                // let (_, _, sum_ele, _) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // Some(sum_ele / ((c2.row as i32  - c1.row as i32+ 1) * (c2.col as i32  - c1.col as i32 + 1)))
                match self.get_avg(c1, c2) {
                    Some(i) => Some(i),
                    None => None,
                    // can directly return self.get_avg since it is also option i32
                }

            }
            Expression::Max(c1, c2)  => {
                // let (_, max_ele, _, _) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // Some(max_ele)
                self.get_max(c1, c2)
            }
            Expression::Min(c1, c2) => {
                // let (min_ele, _, _, _) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // Some(min_ele)
                self.get_min(c1, c2)
            }
            Expression::Sum(c1, c2) => {
                // let (_, _, sum_ele, _) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // Some(sum_ele)
                self.get_sum(c1, c2)
            }
            Expression::Stdev(c1, c2) => {
                // let (_, _, sum_ele, square_ele) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // let area = (c2.row as i32  - c1.row as i32+ 1) * (c2.col as i32  - c1.col as i32 + 1);
                // let avg = sum_ele/area;
                // let sq_avg =( (square_ele/area )as f64).sqrt() as i32;

                // Some(sq_avg - avg)
                self.get_stddev(c1, c2)
            }

            _ => panic!("Unimplemented Expression Matching in get_expr_res!"),
        }
    }

    #[inline]
    fn get_pointer (&self, inp_cell: &Cell) -> usize {
        inp_cell.row as usize * self.col + inp_cell.col as usize
    }

    fn remove_children (&mut self, inp_cell: Cell) {

        
        let expr ;

        if self.exprs.contains_key(&inp_cell) {
            expr = self.exprs.get(&inp_cell).expect("Weird!").clone();
        } else {
            return
        }

       

        match expr {
            Expression::Add(v1, v2)
            | Expression::Mul(v1, v2)
            | Expression::Div(v1, v2)
            | Expression::Sub(v1, v2) => {
                self.remove_children_helper(v1, &inp_cell);
                self.remove_children_helper(v2, &inp_cell);
            }
            
            Expression::Sleep(v) 
            | Expression::Constant(v) => {
                self.remove_children_helper(v, &inp_cell);
            }

            Expression::Avg(c1, c2)
            | Expression::Max(c1, c2)
            | Expression::Min(c1, c2)
            | Expression::Sum(c1, c2)
            | Expression::Stdev(c1, c2) => {
                self.remove_children_helper(Value::Ref(c1), &inp_cell);
                self.remove_children_helper(Value::Ref(c2), &inp_cell);
            },
            _ => panic!("Unimplemented add_children!"),
        }


    }

    fn add_children_helper (&mut self, v: Value, inp_cell: &Cell) {
        match v {
            Value::Num(_) => return,
            Value::Ref(c) => {
                let parent_pointer = self.get_pointer(&c);
                self.spreadsheet[parent_pointer].children.insert(inp_cell.clone());
                ()
            },
        }
    }

    fn remove_children_helper (&mut self, v: Value, inp_cell: &Cell) {
        match v {
            Value::Num(_) => return,
            Value::Ref(c) => {
                let parent_pointer = self.get_pointer(&c);
                self.spreadsheet[parent_pointer].children.remove(inp_cell);
                ()
            },
        }
    }

    fn add_children (&mut self, inp_cell: Cell, expr: Expression) {

        
        match expr {
            Expression::Add(v1, v2)
            | Expression::Mul(v1, v2)
            | Expression::Div(v1, v2)
            | Expression::Sub(v1, v2) => {
                self.add_children_helper(v1, &inp_cell);
                self.add_children_helper(v2, &inp_cell);
            }

            Expression::Sleep(v) 
            | Expression::Constant(v) => {
                self.add_children_helper(v, &inp_cell);
            }

            Expression::Avg(c1, c2)
            | Expression::Max(c1, c2)
            | Expression::Min(c1, c2)
            | Expression::Sum(c1, c2)
            | Expression::Stdev(c1, c2) => {
                self.add_children_helper(Value::Ref(c1), &inp_cell);
                self.add_children_helper(Value::Ref(c2), &inp_cell)
            },
            
            _ => panic!("Unimplemented add_children!"),
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

        match form.expression {
            Expression::Add(Value::Num(_), Value::Num(_)) 
            | Expression::Mul(Value::Num(_), Value::Num(_)) 
            | Expression::Div(Value::Num(_), Value::Num(_)) 
            | Expression::Sub(Value::Num(_), Value::Num(_)) 
            | Expression::Sleep(Value::Num(_))
            | Expression::Constant(Value::Num(_)) => {
                if self.exprs.contains_key(&form.inp_cell) {
                    self.exprs.remove(&form.inp_cell);
                }
                self.val[cell_pointer] = self.get_expr_res(form.expression.clone()).expect("Invalid Expression");
            }

            _ => {
                self.exprs.insert(form.inp_cell.clone(), form.expression.clone());
                self.add_children(form.inp_cell.clone(), form.expression);
                ()
            }
        }

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
                if self.valid[cell_pointer+j as usize] == 1 {
                    print!("{:<10}", "err");
                }
                else {
                    print!("{:<10}", self.val[cell_pointer + j as usize]);
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