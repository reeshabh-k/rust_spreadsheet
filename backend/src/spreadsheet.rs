use crate::input::Col;
use crate::{
    basic::Cell, basic::Expression, basic::Formula, basic::SpreadSheetError, basic::Value,
};

use std::thread;
use std::time::Duration;

use std::{collections::HashMap, collections::HashSet};

#[derive(Clone, Debug)]
struct CellData {
    children: HashSet<Cell>,
}

#[derive(Clone, Debug)]
pub enum CellVal {
    Int_c(i32),
    Str_c(String),
}

pub struct SpreadSheet {
    row_pointer: usize,
    col_pointer: usize,
    row: usize,
    col: usize,
    pub spreadsheet: Vec<CellData>,
    pub val: Vec<CellVal>,
    pub valid: Vec<u8>,
    pub exprs: HashMap<Cell, Expression>,
}

impl SpreadSheet {
    pub fn new(row: usize, col: usize) -> SpreadSheet {
        let default_cell = CellData {
            children: HashSet::new(),
        };
        SpreadSheet {
            row_pointer: 1,
            col_pointer: 1,
            row,
            col,
            spreadsheet: vec![default_cell; (row + 1) * (1 + col)],
            val: vec![CellVal::Int_c(0); (row + 1) * (col + 1)],
            valid: vec![0; (row + 1) * (col + 1)],
            exprs: HashMap::new(),
        }
    }

    fn update_cell(&mut self, cell: Cell) -> CellVal {
        let row = cell.row as usize;
        let col = cell.col as usize;

        let cell_loc = self.col * row + col;
        // let _cell_data = &self.spreadsheet[cell_loc];

        let expr = self.exprs.get(&cell).expect("Weird!");

        match expr {
            Expression::Stringof(s) => {
                self.valid[cell_loc] = 0;
                self.val[cell_loc] = CellVal::Str_c(s.to_string());
                return self.val[cell_loc].clone()
            },
            _ => ()
        }


        let eval_expr = self.get_expr_res(expr.clone());

        match eval_expr {
            None => {
                self.valid[cell_loc] = 1;
                self.val[cell_loc] = CellVal::Str_c("err".to_string())
            },
            Some(cv) => {
                self.valid[cell_loc] = 0;
                self.val[cell_loc] = cv
            },
            
        }
        self.val[cell_loc].clone()
    }

    fn extract_value_num(&self, val: Value) -> Option<i32> {
        match val {
            Value::Num(i) => Some(i),
            Value::Ref(cell) => {
                let cell_point = self.col * (cell.row as usize) + (cell.col as usize);
                if self.valid[cell_point] == 1 {
                    None
                } else {
                    if let CellVal::Int_c(val) = self.val[cell_point] {
                        Some(val)
                    } else {
                        None
                    }
                }
            }
        }
    }

    fn get_sum(&self, c1: Cell, c2: Cell) -> Option<i32> {
        let mut sum = 0;
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell {row : i, col : j})] == 1 {
                    return None;
                }
                if let CellVal::Int_c(val) = self.val[self.get_pointer(&Cell {row: i, col: j})] {
                    sum += val;
                } else {
                    return None;
                }
            }
        }
        Some(sum)
    }

    fn get_avg(&self, c1: Cell, c2: Cell) -> Option<i32> {
        let mut sum = 0;
        let mut cnt = 0;
        // sum = self.spreadsheet[self.get_pointer(&Cell {row : c1.row, col : c1.col})].val;
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell { row: i, col: j })] == 1 {
                    return None;
                }
                if let CellVal::Int_c(val) = self.val[self.get_pointer(&Cell {row: i, col: j})] {
                    sum += val;
                    cnt += 1;
                } else {
                    return None;
                }
            }
        }
        let mean = sum / cnt;
        Some(mean)
    }

    fn get_max(&self, c1: Cell, c2: Cell) -> Option<i32> {
        let mut max;
        // let mut min;
        if let CellVal::Int_c(maxi) = self.val[self.get_pointer(&Cell {
            row: c1.row,
            col: c1.col,
        })] {
            max = maxi;
        } else {
            max = 0;
            return None;
        }
        // min = self.spreadsheet[self.get_pointer(&Cell {row : c1.row, col : c1.col})].val;
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell { row: i, col: j })] == 1 {
                    return None;
                }
                if let CellVal::Int_c(val) = self.val[self.get_pointer(&Cell { row: i, col: j })] {
                    if val > max {
                        max = val;
                    }
                } else {
                    return None;
                }
            }
        }
        Some(max)
    }

    fn get_min(&self, c1: Cell, c2: Cell) -> Option<i32> {
        // let mut max;
        let mut min;
        // max = self.spreadsheet[self.get_pointer(&Cell {row : c1.row, col : c1.col})].val;
        if let CellVal::Int_c(mini) = self.val[self.get_pointer(&Cell {
            row: c1.row,
            col: c1.col,
        })] {
            min = mini
        } else {
            min = 0;
            return None;
        }
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell { row: i, col: j })] == 1 {
                    return None;
                }
                
                
                if let CellVal::Int_c(val) = self.val[self.get_pointer(&Cell { row: i, col: j })] {
                    if val < min {
                        min = val;
                    }
                } else {
                    return None;
                }
            }
        }
        Some(min)
    }

    fn get_stddev(&self, c1: Cell, c2: Cell) -> Option<i32> {
        let mut mean = 0;
        let mut variance = 0.0;
        let mut cnt = 0;
        let mut temp: i32;
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell { row: i, col: j })] == 1 {
                    return None;
                }

                if let CellVal::Int_c(val) = self.val[self.get_pointer(&Cell { row: i, col: j })] {
                    mean += val;
                    cnt += 1;
                } else {
                    return None;
                } 
                
            }
        }
        mean /= cnt;

        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if let CellVal::Int_c(temp) = self.val[self.get_pointer(&Cell { row: i, col: j })] {
                    variance += ((temp - mean) * (temp - mean)) as f64;
                } else {
                    return None;
                }
            }
        }
        variance /= cnt as f64;
        let stddev = variance.sqrt();
        Some(stddev.round() as i32)
    }

    fn extract_constant_val(&self, val: Value) -> Option<CellVal> {
        match val {
            Value::Num(i) => Some(CellVal::Int_c(i)),
            Value::Ref(cell) => {
                let cell_point = self.col * (cell.row as usize) + (cell.col as usize);
                if self.valid[cell_point] == 1 {
                    None
                } else {
                    Some(self.val[cell_point].clone())
                }
            }
        }
    }

    fn get_expr_res(&self, expr: Expression) -> Option<CellVal> {
        match expr {
            Expression::Add(v1, v2) => {
                Some(CellVal::Int_c(self.extract_value_num(v1)? + self.extract_value_num(v2)?))
            }
            Expression::Mul(v1, v2) => {
                Some(CellVal::Int_c(self.extract_value_num(v1)? * self.extract_value_num(v2)?))
            }
            Expression::Div(v1, v2) => {
                let denom = self.extract_value_num(v2)?;
                if denom == 0 {
                    None
                } else {
                    Some(CellVal::Int_c(self.extract_value_num(v1)? / denom))
                }
            }
            Expression::Sub(v1, v2) => {
                Some(CellVal::Int_c(self.extract_value_num(v1)? - self.extract_value_num(v2)?))
            }
            Expression::Constant(v) => {
                self.extract_constant_val(v)
            },
            Expression::Sleep(v) => {
                let sleep_time = self.extract_value_num(v)?;
                thread::sleep(Duration::from_secs(sleep_time as u64));
                Some(CellVal::Int_c(sleep_time))
            }

            Expression::Avg(c1, c2) => {
                // let (_, _, sum_ele, _) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // Some(sum_ele / ((c2.row as i32  - c1.row as i32+ 1) * (c2.col as i32  - c1.col as i32 + 1)))
                Some(CellVal::Int_c(self.get_avg(c1, c2)?))
            }
            Expression::Max(c1, c2) => {
                // let (_, max_ele, _, _) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // Some(max_ele)
                Some(CellVal::Int_c(self.get_max(c1, c2)?))
            }
            Expression::Min(c1, c2) => {
                // let (min_ele, _, _, _) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // Some(min_ele)
                Some(CellVal::Int_c(self.get_min(c1, c2)?))
            }
            Expression::Sum(c1, c2) => {
                // let (_, _, sum_ele, _) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // Some(sum_ele)
                Some(CellVal::Int_c(self.get_sum(c1, c2)?))
            }
            Expression::Stdev(c1, c2) => {
                // let (_, _, sum_ele, square_ele) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // let area = (c2.row as i32  - c1.row as i32+ 1) * (c2.col as i32  - c1.col as i32 + 1);
                // let avg = sum_ele/area;
                // let sq_avg =( (square_ele/area )as f64).sqrt() as i32;

                // Some(sq_avg - avg)
                Some(CellVal::Int_c(self.get_stddev(c1, c2)?))
            }

            _ => panic!("Unimplemented Expression Matching in get_expr_res!"),
        }
    }

    #[inline]
    pub fn get_pointer(&self, inp_cell: &Cell) -> usize {
        inp_cell.row as usize * self.col + inp_cell.col as usize
    }

    fn remove_children(&mut self, inp_cell: Cell) {
        let expr;

        if self.exprs.contains_key(&inp_cell) {
            expr = self.exprs.get(&inp_cell).expect("Weird!").clone();
        } else {
            return;
        }

        match expr {
            Expression::Add(v1, v2)
            | Expression::Mul(v1, v2)
            | Expression::Div(v1, v2)
            | Expression::Sub(v1, v2) => {
                self.remove_children_helper(v1, &inp_cell);
                self.remove_children_helper(v2, &inp_cell);
            }

            Expression::Sleep(v) | Expression::Constant(v) => {
                self.remove_children_helper(v, &inp_cell);
            }

            Expression::Avg(c1, c2)
            | Expression::Max(c1, c2)
            | Expression::Min(c1, c2)
            | Expression::Sum(c1, c2)
            | Expression::Stdev(c1, c2) => {
                for i in c1.row..=c2.row {
                    for j in c1.col..= c2.col {
                        let c = Cell { row: i, col: j};
                        self.add_children_helper(Value::Ref(c), &inp_cell);
                    }
                }
            },
            | Expression::Stringof(_) => (),
            _ => panic!("Unimplemented add_children!"),
        }
    }

    fn add_children_helper(&mut self, v: Value, inp_cell: &Cell) {
        match v {
            Value::Num(_) => (),
            Value::Ref(c) => {
                let parent_pointer = self.get_pointer(&c);
                self.spreadsheet[parent_pointer].children.insert(*inp_cell);
            }
        }
    }

    fn remove_children_helper(&mut self, v: Value, inp_cell: &Cell) {
        match v {
            Value::Num(_) => (),
            Value::Ref(c) => {
                let parent_pointer = self.get_pointer(&c);
                self.spreadsheet[parent_pointer].children.remove(inp_cell);
            }
        }
    }

    fn add_children(&mut self, inp_cell: Cell, expr: Expression) {
        match expr {
            Expression::Add(v1, v2)
            | Expression::Mul(v1, v2)
            | Expression::Div(v1, v2)
            | Expression::Sub(v1, v2) => {
                self.add_children_helper(v1, &inp_cell);
                self.add_children_helper(v2, &inp_cell);
            }

            Expression::Sleep(v) | Expression::Constant(v) => {
                self.add_children_helper(v, &inp_cell);
            }

            Expression::Avg(c1, c2)
            | Expression::Max(c1, c2)
            | Expression::Min(c1, c2)
            | Expression::Sum(c1, c2)
            | Expression::Stdev(c1, c2) => {
                for i in c1.row..=c2.row {
                    for j in c1.col..= c2.col {
                        let c = Cell { row: i, col: j};
                        self.add_children_helper(Value::Ref(c), &inp_cell);
                    }
                }
                
            },
            | Expression::Stringof(_) => (),

            _ => panic!("Unimplemented add_children!"),
        }
    }

    pub fn call_formula_api (&mut self, form: Option<Formula>) -> (String, String, String){
        let form = match form {
            None => return (String::from("IV"),String::new(),String::new()),
            Some(valid_form) => valid_form,
        };
        match form.expression {
            Expression::Quit 
            |Expression::Disable 
            |Expression::Enable 
            |Expression::ScrollDown 
            |Expression::ScrollUp 
            |Expression::ScrollRight 
            |Expression::ScrollLeft 
            |Expression::ScrollTo(_)  => {
                return (String::new(), String::new(), String::new())
            },

            Expression::Avg(c1, c2)
            | Expression::Max(c1, c2)
            | Expression::Min(c1, c2)
            | Expression::Sum(c1, c2) 
            | Expression::Stdev(c1, c2) => {
                if c1.row > c2.row || c1.col > c2.col {
                    return (String::from("IV"), String::new(), String::new());
                }
            }
            _ => (),
        }
        if self.check_cycle(form.clone()) {
            return (String::from("CY"), String::new(), String::new());
        }
        self.remove_children(form.inp_cell);
        self.exprs.insert(form.inp_cell, form.expression.clone());
        self.add_children(form.inp_cell, form.expression);

        let (x, y, z) = self.update_children_api(form.inp_cell);
        (x, y, z)
    }

    fn update_children_api(&mut self, inp_cell: Cell) -> (String, String, String) {
        let mut cell_counts: HashMap<Cell, u32> = HashMap::new();
        cell_counts.insert(inp_cell, 0);

        let mut stack: Vec<Cell> = Vec::new();

        stack.push(inp_cell);

        let mut k = 0;

        while !stack.is_empty() {
            k += 1;
            let top_cell = stack.pop().expect("Stack is empty!");

            let cell_pointer = self.get_pointer(&top_cell);

            for i in self.spreadsheet[cell_pointer].children.iter() {
                if !cell_counts.contains_key(i) {
                    stack.push(*i);
                }
                cell_counts.insert(*i, k);
            }
        }

        let mut sorted_vec: Vec<(Cell, u32)> = cell_counts.into_iter().collect();

        sorted_vec.sort_by(|a, b| a.1.cmp(&b.1));

        let mut x = String::new();
        let mut y = String::new();
        let mut z = String::new();


        for (i, _) in sorted_vec.iter() {
            let num = self.update_cell(*i);
            x.push_str((format!("{}{} ", String::from_utf8_lossy(&Col::from_num(i.col as u32).expect("Error Converting Num to Col").0), i.row)).as_str());
            y.push_str((format!("{} ", i.col)).as_str());
            match num {
                CellVal::Int_c(v) => z.push_str((format!("{}|", v)).as_str()),
                CellVal::Str_c(s) => z.push_str((format!("{}|", s)).as_str()),
            }
            
        }
        (x, y, z)
    }

    fn belongs_to_expression(&self, expr: &Expression, c: Cell) -> bool {
        let val = Value::Ref(c);

        match expr {
            Expression::Add(v1, v2)
            | Expression::Div(v1, v2)
            | Expression::Mul(v1, v2)
            | Expression::Sub(v1, v2) => *v1 == val || *v2 == val,

            Expression::Sleep(v) | Expression::Constant(v) => *v == val,

            Expression::Avg(c1, c2)
            | Expression::Max(c1, c2)
            | Expression::Min(c1, c2)
            | Expression::Sum(c1, c2)
            | Expression::Stdev(c1, c2) => {
                c.row >= c1.row && c.col >= c1.col && c.row <= c2.row && c.col <= c2.col
            },

            Expression::Stringof(_) => false,

            _ => panic!("Unimplemented belongs_to_expression!"),
        }
    }

    fn check_cycle(&self, form: Formula) -> bool {
        let inp_cell = form.inp_cell;
        let expr = form.expression;

        if self.belongs_to_expression(&expr, inp_cell) {
            return true;
        }

        let mut visited: HashSet<Cell> = HashSet::new();
        visited.insert(inp_cell);

        let mut stack: Vec<Cell> = Vec::new();
        stack.push(inp_cell);

        while let Some(top_cell) = stack.pop() {
            let cell_pointer = self.get_pointer(&top_cell);

            for i in self.spreadsheet[cell_pointer].children.iter() {
                if self.belongs_to_expression(&expr, *i) {
                    return true;
                }
                if !visited.contains(i) {
                    stack.push(*i);
                    visited.insert(*i);
                }
            }
        }
        false
    }

    pub fn print_sheet(&self) {
        let width = 10.min(self.col as u32 - self.col_pointer as u32 + 1);
        let length = 10.min(self.row as u32 - self.row_pointer as u32 + 1);
        print!("          ");
        for i in 0..width {
            let col = Col::from_num(self.col_pointer as u32 + i)
                .expect("Col Pointer is at Invalid Location");
            print!("{:<10}", col.as_str());
        }
        println!();
        for i in 0..length {
            print!("{:<10}", self.row_pointer + i as usize);
            let cell_pointer = self.col * (i as usize + self.row_pointer) + self.col_pointer;
            for j in 0..width {
                if self.valid[cell_pointer + j as usize] == 1 {
                    print!("{:<10}", "err");
                } else {
                    match self.val[cell_pointer + j as usize].clone() {
                        CellVal::Int_c(i) => print!("{:<10}", i),
                        CellVal::Str_c(s) => print!("{:<10}", s),
                    }
                }
            }
            println!();
        }
    }
}
