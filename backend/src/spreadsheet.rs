use crate::input::Col;
use crate::{basic::Cell, basic::Expression, basic::Formula, basic::Value};

use std::thread;
use std::time::Duration;

use std::{collections::HashMap, collections::HashSet};

/// A structure that holds information about a cell, such as its children (dependent cells).
/// This helps in managing dependencies between cells when formulas are used.
#[allow(private_interfaces)]
#[derive(Clone, Debug)]
pub struct CellData {
    children: HashSet<Cell>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CellVal {
    IntC(i32),
    StrC(String),
}
/// A structure representing the entire spreadsheet, holding the data for all cells.
/// It supports expressions, formulas, and dependency tracking between cells.
#[allow(dead_code)]
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

#[allow(dead_code)]
impl SpreadSheet {
    /// Creates a new `SpreadSheet` with the given number of rows and columns.
    ///
    /// # Parameters
    /// - `row`: The number of rows in the spreadsheet.
    /// - `col`: The number of columns in the spreadsheet.
    ///
    /// # Returns
    /// A `SpreadSheet` instance initialized with the given dimensions.
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
            val: vec![CellVal::IntC(0); (row + 1) * (col + 1)],
            valid: vec![0; (row + 1) * (col + 1)],
            exprs: HashMap::new(),
        }
    }

    /// Updates the value of a cell based on its expression.
    ///
    /// If the cell has an expression, it will evaluate the expression and update
    /// the value of the cell accordingly.
    ///
    /// # Parameters
    /// - `cell`: The cell to update.
    fn update_cell(&mut self, cell: Cell) -> CellVal {
        let row = cell.row as usize;
        let col = cell.col as usize;

        let cell_loc = self.col * row + col;
        // let _cell_data = &self.spreadsheet[cell_loc];

        let expr = self.exprs.get(&cell).expect("Weird!");

        if let Expression::Stringof(s) = expr {
            self.valid[cell_loc] = 0;
            self.val[cell_loc] = CellVal::StrC(s.to_string());
            return self.val[cell_loc].clone();
        }

        let eval_expr = self.get_expr_res(expr.clone());

        match eval_expr {
            None => {
                self.valid[cell_loc] = 1;
                self.val[cell_loc] = CellVal::StrC("err".to_string())
            }
            Some(cv) => {
                self.valid[cell_loc] = 0;
                self.val[cell_loc] = cv
            }
        }
        self.val[cell_loc].clone()
    }

    /// Extracts a numerical value from a `Value` enum.
    ///
    /// # Parameters
    /// - `val`: The value to extract.
    ///
    /// # Returns
    /// An optional `i32` value. If the value is a reference to another cell, it will return `None`.
    fn extract_value_num(&self, val: Value) -> Option<i32> {
        match val {
            Value::Num(i) => Some(i),
            Value::Ref(cell) => {
                let cell_point = self.col * (cell.row as usize) + (cell.col as usize);
                if self.valid[cell_point] == 1 {
                    None
                } else if let CellVal::IntC(val) = self.val[cell_point] {
                    Some(val)
                } else {
                    None
                }
            }
        }
    }

    /// Calculates the sum of a range of cells.
    ///
    /// # Parameters
    /// - `c1`: The top-left corner of the range.
    /// - `c2`: The bottom-right corner of the range.
    ///
    /// # Returns
    /// An optional sum of the values in the specified range.
    fn get_sum(&self, c1: Cell, c2: Cell) -> Option<i32> {
        let mut sum = 0;
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell { row: i, col: j })] == 1 {
                    return None;
                }
                if let CellVal::IntC(val) = self.val[self.get_pointer(&Cell { row: i, col: j })] {
                    sum += val;
                } else {
                    return None;
                }
            }
        }
        Some(sum)
    }

    /// Calculates the average of a range of cells.
    ///
    /// # Parameters
    /// - `c1`: The top-left corner of the range.
    /// - `c2`: The bottom-right corner of the range.
    ///
    /// # Returns
    /// An optional average of the values in the specified range.
    fn get_avg(&self, c1: Cell, c2: Cell) -> Option<i32> {
        let mut sum = 0;
        let mut cnt = 0;
        // sum = self.spreadsheet[self.get_pointer(&Cell {row : c1.row, col : c1.col})].val;
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell { row: i, col: j })] == 1 {
                    return None;
                }
                if let CellVal::IntC(val) = self.val[self.get_pointer(&Cell { row: i, col: j })] {
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

    /// Finds the maximum value in a range of cells.
    ///
    /// # Parameters
    /// - `c1`: The top-left corner of the range.
    /// - `c2`: The bottom-right corner of the range.
    ///
    /// # Returns
    /// An optional maximum value of the specified range.
    fn get_max(&self, c1: Cell, c2: Cell) -> Option<i32> {
        let mut max;
        // let mut min;
        if let CellVal::IntC(maxi) = self.val[self.get_pointer(&Cell {
            row: c1.row,
            col: c1.col,
        })] {
            max = maxi;
        } else {
            // max = 0;
            return None;
        }
        // min = self.spreadsheet[self.get_pointer(&Cell {row : c1.row, col : c1.col})].val;
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell { row: i, col: j })] == 1 {
                    return None;
                }
                if let CellVal::IntC(val) = self.val[self.get_pointer(&Cell { row: i, col: j })] {
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

    /// Finds the minimum value in a range of cells.
    ///
    /// # Parameters
    /// - `c1`: The top-left corner of the range.
    /// - `c2`: The bottom-right corner of the range.
    ///
    /// # Returns
    /// An optional minimum value of the specified range.
    fn get_min(&self, c1: Cell, c2: Cell) -> Option<i32> {
        // let mut max;
        let mut min;
        // max = self.spreadsheet[self.get_pointer(&Cell {row : c1.row, col : c1.col})].val;
        if let CellVal::IntC(mini) = self.val[self.get_pointer(&Cell {
            row: c1.row,
            col: c1.col,
        })] {
            min = mini
        } else {
            // min = 0;
            return None;
        }
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell { row: i, col: j })] == 1 {
                    return None;
                }

                if let CellVal::IntC(val) = self.val[self.get_pointer(&Cell { row: i, col: j })] {
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

    /// Finds the stdev value in a range of cells.
    ///
    /// # Parameters
    /// - `c1`: The top-left corner of the range.
    /// - `c2`: The bottom-right corner of the range.
    ///
    /// # Returns
    /// An optional stdev value of the specified range.
    fn get_stddev(&self, c1: Cell, c2: Cell) -> Option<i32> {
        let mut mean = 0;
        let mut variance = 0.0;
        let mut cnt = 0;

        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell { row: i, col: j })] == 1 {
                    return None;
                }

                if let CellVal::IntC(val) = self.val[self.get_pointer(&Cell { row: i, col: j })] {
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
                if let CellVal::IntC(temp) = self.val[self.get_pointer(&Cell { row: i, col: j })] {
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
            Value::Num(i) => Some(CellVal::IntC(i)),
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

    /// Evaluates the result of an expression.
    ///
    /// # Parameters
    /// - `expr`: The expression to evaluate.
    ///
    /// # Returns
    /// The evaluated result as an `Option<i32>`. Returns `None` for invalid or undefined expressions (e.g., division by zero).
    ///
    /// # Panics
    /// This function will panic if the expression is not supposed to be called.    
    fn get_expr_res(&self, expr: Expression) -> Option<CellVal> {
        match expr {
            Expression::Add(v1, v2) => Some(CellVal::IntC(
                self.extract_value_num(v1)? + self.extract_value_num(v2)?,
            )),
            Expression::Mul(v1, v2) => Some(CellVal::IntC(
                self.extract_value_num(v1)? * self.extract_value_num(v2)?,
            )),
            Expression::Div(v1, v2) => {
                let denom = self.extract_value_num(v2)?;
                if denom == 0 {
                    None
                } else {
                    Some(CellVal::IntC(self.extract_value_num(v1)? / denom))
                }
            }
            Expression::Sub(v1, v2) => Some(CellVal::IntC(
                self.extract_value_num(v1)? - self.extract_value_num(v2)?,
            )),
            Expression::Constant(v) => self.extract_constant_val(v),
            Expression::Sleep(v) => {
                let sleep_time = self.extract_value_num(v)?;
                thread::sleep(Duration::from_secs(sleep_time as u64));
                Some(CellVal::IntC(sleep_time))
            }

            Expression::Avg(c1, c2) => {
                // let (_, _, sum_ele, _) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // Some(sum_ele / ((c2.row as i32  - c1.row as i32+ 1) * (c2.col as i32  - c1.col as i32 + 1)))
                Some(CellVal::IntC(self.get_avg(c1, c2)?))
            }
            Expression::Max(c1, c2) => {
                // let (_, max_ele, _, _) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // Some(max_ele)
                Some(CellVal::IntC(self.get_max(c1, c2)?))
            }
            Expression::Min(c1, c2) => {
                // let (min_ele, _, _, _) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // Some(min_ele)
                Some(CellVal::IntC(self.get_min(c1, c2)?))
            }
            Expression::Sum(c1, c2) => {
                // let (_, _, sum_ele, _) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // Some(sum_ele)
                Some(CellVal::IntC(self.get_sum(c1, c2)?))
            }
            Expression::Stdev(c1, c2) => {
                // let (_, _, sum_ele, square_ele) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // let area = (c2.row as i32  - c1.row as i32+ 1) * (c2.col as i32  - c1.col as i32 + 1);
                // let avg = sum_ele/area;
                // let sq_avg =( (square_ele/area )as f64).sqrt() as i32;

                // Some(sq_avg - avg)
                Some(CellVal::IntC(self.get_stddev(c1, c2)?))
            }

            _ => panic!("Unimplemented Expression Matching in get_expr_res!"),
        }
    }

    /// Helper function to get a pointer (index) for a given cell.
    ///
    /// # Parameters
    /// - `cell`: The cell for which to get the pointer.
    ///
    /// # Returns
    /// An index in the vector representing the cell.
    #[inline]
    pub fn get_pointer(&self, inp_cell: &Cell) -> usize {
        inp_cell.row as usize * self.col + inp_cell.col as usize
    }

    /// Removes all children (dependencies) of a given cell from the spreadsheet.
    ///
    /// # Parameters
    /// - `inp_cell`: The cell for which to remove children.
    ///
    /// # Returns
    /// Nothing. The children of the specified cell are removed.
    fn remove_children(&mut self, inp_cell: Cell) {
        let expr = if self.exprs.contains_key(&inp_cell) {
            self.exprs.get(&inp_cell).expect("Weird!").clone()
        } else {
            return;
        };

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
                    for j in c1.col..=c2.col {
                        let c = Cell { row: i, col: j };
                        self.remove_children_helper(Value::Ref(c), &inp_cell);
                    }
                }
            }
            Expression::Stringof(_) => (),
            _ => panic!("Unimplemented add_children!"),
        }
    }

    /// Helper function to add a cell as a child to another cell.
    ///
    /// # Parameters
    /// - `v`: The value associated with the cell (either a number or a reference to another cell).
    /// - `inp_cell`: The cell to which the value will be added as a child.
    fn add_children_helper(&mut self, v: Value, inp_cell: &Cell) {
        match v {
            Value::Num(_) => (),
            Value::Ref(c) => {
                let parent_pointer = self.get_pointer(&c);
                self.spreadsheet[parent_pointer].children.insert(*inp_cell);
            }
        }
    }

    /// Helper function to remove a cell as a child from another cell.
    ///
    /// # Parameters
    /// - `v`: The value associated with the cell (either a number or a reference to another cell).
    /// - `inp_cell`: The cell to be removed as a child.
    fn remove_children_helper(&mut self, v: Value, inp_cell: &Cell) {
        match v {
            Value::Num(_) => (),
            Value::Ref(c) => {
                let parent_pointer = self.get_pointer(&c);
                self.spreadsheet[parent_pointer].children.remove(inp_cell);
            }
        }
    }

    /// Adds a new child (dependency) to a cell.
    ///
    /// # Parameters
    /// - `inp_cell`: The cell for which to add a dependency.
    /// - `expr`: The expression that defines the dependency.
    ///
    /// # Returns
    /// Nothing. The child (dependency) is added to the specified cell.
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
                    for j in c1.col..=c2.col {
                        let c = Cell { row: i, col: j };
                        self.add_children_helper(Value::Ref(c), &inp_cell);
                    }
                }
            }
            Expression::Stringof(_) => (),

            _ => panic!("Unimplemented add_children!"),
        }
    }

    /// Handles formula calls, processes different expressions, updates spreadsheet state, and handles cycles.
    ///
    /// # Arguments
    /// * `form`: An `Option<Formula>`, where `Some(valid_form)` contains a valid formula and `None` indicates invalid input.
    ///
    /// # Returns
    /// * A tuple of strings in which the changed cells (row and column) with their modified values is stored with spaces in between, for ease of formatting to json.
    ///
    /// This function processes various spreadsheet expressions, handles scrolling, and evaluates cell expressions. 
    /// It also ensures that cyclic dependencies are checked to prevent infinite loops.
    pub fn call_formula_api(&mut self, form: Option<Formula>) -> (String, String, String) {
        let form = match form {
            None => return (String::from("IV"), String::new(), String::new()),
            Some(valid_form) => valid_form,
        };
        match form.expression {
            Expression::Quit
            | Expression::Disable
            | Expression::Enable
            | Expression::ScrollDown
            | Expression::ScrollUp
            | Expression::ScrollRight
            | Expression::ScrollLeft
            | Expression::ScrollTo(_) => return (String::new(), String::new(), String::new()),

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

    /// Updates all children cells for a given cell `inp_cell` based on its dependencies and expression results.
    ///
    /// # Arguments
    /// * `inp_cell`: The `Cell` whose children need to be updated.
    ///
    /// # Returns
    /// Returns a tuple of strings which contains the changed cells and their modified values separated by spaces
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
            x.push_str(
                (format!(
                    "{}{} ",
                    String::from_utf8_lossy(
                        &Col::from_num(i.col as u32)
                            .expect("Error Converting Num to Col")
                            .0
                    ),
                    i.row
                ))
                .as_str(),
            );
            y.push_str((format!("{} ", i.col)).as_str());
            match num {
                CellVal::IntC(v) => z.push_str((format!("{v}|")).as_str()),
                CellVal::StrC(s) => z.push_str((format!("{s}|")).as_str()),
            }
        }
        (x, y, z)
    }

    /// Determines if a given `expr` belongs to a given cell `c` based on the expression type and operands.
    ///
    /// # Arguments
    /// * `expr`: A reference to the `Expression` to check.
    /// * `c`: The `Cell` to check for membership in the expression.
    ///
    /// # Returns
    /// * `bool`: Returns `true` if the expression belongs to the cell, `false` otherwise.
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
            }

            Expression::Stringof(_) => false,

            _ => panic!("Unimplemented belongs_to_expression!"),
        }
    }

    /// Checks if the given formula `form` creates a cyclic dependency in the spreadsheet.
    ///
    /// # Arguments
    /// * `form`: The `Formula` to check for cycles.
    ///
    /// # Returns
    /// * `bool`: Returns `true` if the formula introduces a cycle, otherwise `false`.
    ///
    /// This function traverses the dependencies of a given formula to check for cyclic references, 
    /// ensuring that the spreadsheet doesn't enter an infinite loop of formula evaluations.
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

    /// Prints the current state of the spreadsheet, showing the values and headers for the visible range.
    ///
    /// This function prints a portion of the spreadsheet, including the column headers and row values, 
    /// as well as handling cases where cells are invalid or contain errors.
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
                        CellVal::IntC(i) => print!("{i:<10}"),
                        CellVal::StrC(s) => print!("{s:<10}"),
                    }
                }
            }
            println!();
        }
    }
}


#[cfg(test)]

mod spreadsheet_tests {

    use super::*;
    use crate::basic::Expression;

    #[test]
    fn test_spreadsheet() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        assert_eq!(ss.val[11], CellVal::IntC(5));
        assert_eq!(ss.valid[11], 0);
    }
    #[test]
    fn test_spreadsheet_add() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Add(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(8));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_spreadsheet_div() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Div(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(2)),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(2));
        assert_eq!(ss.valid[22], 0);
    }

    #[test]
    fn test_spreadsheet_mul() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Mul(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(15));
        assert_eq!(ss.valid[22], 0);
    }

    #[test]
    fn test_spreadsheet_sub() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Sub(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(2));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_spreadsheet_avg() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Avg(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test] 
    fn test_spreadsheet_max() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Max(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_spreadsheet_min() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Min(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_spreadsheet_stdev() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Stdev(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_spreadsheet_sum() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Sum(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_spreadsheet_stringof() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Stringof("Hello".to_string()),
        }));
        assert_eq!(ss.val[11], CellVal::StrC("Hello".to_string()));
        assert_eq!(ss.valid[11], 0);
    }
    #[test]
    fn test_spreadsheet_sleep() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(1)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Sleep(Value::Num(1)),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(1));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_spreadsheet_cycle() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Add(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 3, col: 3 },
            expression: Expression::Add(Value::Ref(Cell { row: 2, col: 2 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[33], CellVal::IntC(11));
        assert_eq!(ss.valid[33], 0);
    }
    #[test]
    fn test_spreadsheet_cycle_invalid() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Add(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 3, col: 3 },
            expression: Expression::Add(Value::Ref(Cell { row: 2, col: 2 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[33], CellVal::IntC(11));
        assert_eq!(ss.valid[33], 0);
    }
    #[test]
    fn test_spreadsheet_cycle_invalid_2() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Add(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 3, col: 3 },
            expression: Expression::Add(Value::Ref(Cell { row: 2, col: 2 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[33], CellVal::IntC(11));
        assert_eq!(ss.valid[33], 0);
    }
    #[test]
    fn test_spreadsheet_cycle_invalid_3() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Add(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 3, col: 3 },
            expression: Expression::Add(Value::Ref(Cell { row: 2, col: 2 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[33], CellVal::IntC(11));
        assert_eq!(ss.valid[33], 0);
    }
    #[test]
    fn test_spreadsheet_cycle_invalid_4() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Add(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 3, col: 3 },
            expression: Expression::Add(Value::Ref(Cell { row: 2, col: 2 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[33], CellVal::IntC(11));
        assert_eq!(ss.valid[33], 0);
    }

    #[test]
    fn test_add_children() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Add(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(8));
        assert_eq!(ss.valid[22], 0);
        assert_eq!(ss.spreadsheet[11].children.len(), 1);
        assert_eq!(ss.spreadsheet[11].children.contains(&Cell { row: 2, col: 2 }), true);
    }
    #[test]
    fn test_remove_children() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Add(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(8));
        assert_eq!(ss.valid[22], 0);
        assert_eq!(ss.spreadsheet[11].children.len(), 1);
        assert_eq!(ss.spreadsheet[11].children.contains(&Cell { row: 2, col: 2 }), true);
        ss.remove_children(Cell { row: 2, col: 2 });
        assert_eq!(ss.spreadsheet[11].children.len(), 0);
    }
    #[test]
    fn test_update_children_api() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Add(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(8));
        assert_eq!(ss.valid[22], 0);
        assert_eq!(ss.spreadsheet[11].children.len(), 1);
        assert_eq!(ss.spreadsheet[11].children.contains(&Cell { row: 2, col: 2 }), true);
    }
    #[test]
    fn test_check_cycle() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Add(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(8));
        assert_eq!(ss.valid[22], 0);
        assert_eq!(ss.spreadsheet[11].children.len(), 1);
        assert_eq!(ss.spreadsheet[11].children.contains(&Cell { row: 2, col: 2 }), true);
    }
    #[test]
    fn test_check_cycle_invalid() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Add(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(8));
        assert_eq!(ss.valid[22], 0);
        assert_eq!(ss.spreadsheet[11].children.len(), 1);
        assert_eq!(ss.spreadsheet[11].children.contains(&Cell { row: 2, col: 2 }), true);
    }
    #[test]
    fn test_get_avg() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        let (x, _, _) = ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Avg(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(x, "CY".to_string());
    }

    #[test]
    fn test_get_max() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Max(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_get_min() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Min(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_extract_value_num() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        assert_eq!(ss.extract_value_num(Value::Num(5)), Some(5));
        assert_eq!(ss.extract_value_num(Value::Ref(Cell { row: 1, col: 1 })), Some(5));
        assert_eq!(ss.extract_value_num(Value::Ref(Cell { row: 2, col: 2 })), Some(0));
    }

    #[test]
    fn test_extract_constant_val() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        assert_eq!(ss.extract_constant_val(Value::Num(5)), Some(CellVal::IntC(5)));
        assert_eq!(
            ss.extract_constant_val(Value::Ref(Cell { row: 1, col: 1 })),
            Some(CellVal::IntC(5))
        );
        assert_eq!(
            ss.extract_constant_val(Value::Ref(Cell { row: 2, col: 2 })),
            Some(CellVal::IntC(0))
        );
    }
    #[test]
    fn test_get_expr_res() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        assert_eq!(
            ss.get_expr_res(Expression::Add(Value::Num(5), Value::Num(3))),
            Some(CellVal::IntC(8))
        );
        assert_eq!(
            ss.get_expr_res(Expression::Mul(Value::Num(5), Value::Num(3))),
            Some(CellVal::IntC(15))
        );
        assert_eq!(
            ss.get_expr_res(Expression::Div(Value::Num(5), Value::Num(2))),
            Some(CellVal::IntC(2))
        );
        assert_eq!(
            ss.get_expr_res(Expression::Sub(Value::Num(5), Value::Num(3))),
            Some(CellVal::IntC(2))
        );
    }
    #[test]
    fn test_get_pointer() {
        let ss = SpreadSheet::new(10, 10);
        assert_eq!(ss.get_pointer(&Cell { row: 1, col: 1 }), 11);
        assert_eq!(ss.get_pointer(&Cell { row: 2, col: 2 }), 22);
        assert_eq!(ss.get_pointer(&Cell { row: 3, col: 3 }), 33);
    }

    
    
    #[test]
    fn test_print_sheet() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.print_sheet();
    }

    #[test]
    fn test_get_sum() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Sum(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }

    #[test]
    fn test_get_stddev() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Stdev(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }

    #[test]
    fn test_get_stringof() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Stringof("Hello".to_string()),
        }));
        assert_eq!(ss.val[11], CellVal::StrC("Hello".to_string()));
        assert_eq!(ss.valid[11], 0);
    }

    #[test]
    fn test_get_sleep() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Sleep(Value::Num(1)),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(1));
        assert_eq!(ss.valid[22], 0);
    }

    #[test]
    fn test_get_disable() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Disable,
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }

    #[test]
    fn test_get_enable() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Enable,
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_get_scroll_down() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::ScrollDown,
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_get_scroll_up() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::ScrollUp,
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_get_scroll_right() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::ScrollRight,
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_get_scroll_left() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::ScrollLeft,
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_get_scroll_to() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::ScrollTo(Cell { row: 3, col: 3 }),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_get_scroll_to_invalid() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::ScrollTo(Cell { row: 3, col: 3 }),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }
    #[test]
    fn test_get_scroll_to_invalid_2() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::ScrollTo(Cell { row: 3, col: 3 }),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(0));
        assert_eq!(ss.valid[22], 0);
    }

    #[test]
    fn test_get_avg2() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_avg(Cell{row:0, col:0}, Cell{row:1, col:1});
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_get_sum2() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_sum(Cell{row:0, col:0}, Cell{row:1, col:1});
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_get_max2() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_max(Cell{row:0, col:0}, Cell{row:1, col:1});
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_get_min2() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_min(Cell{row:0, col:0}, Cell{row:1, col:1});
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_get_stdev2() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_stddev(Cell{row:0, col:0}, Cell{row:1, col:1});
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }

    #[test]
    fn test_get_expr_res2() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_expr_res(Expression::Add(Value::Num(5), Value::Num(3)));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_get_constant_val2() {
        let ss = SpreadSheet::new(10, 10);
        ss.extract_constant_val(Value::Num(5));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_belongs_to_expression2() {
        let ss = SpreadSheet::new(10, 10);
        ss.belongs_to_expression(&Expression::Constant(Value::Num(5)), Cell { row: 1, col: 1 });
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_belongs_to_expression3() {
        let ss = SpreadSheet::new(10, 10);
        ss.belongs_to_expression(&Expression::Constant(Value::Num(5)), Cell { row: 1, col: 1 });
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_get_expr_res3() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_expr_res(Expression::Sub(Value::Num(5), Value::Num(3)));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_get_expr_res4() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_expr_res(Expression::Div(Value::Num(5), Value::Num(2)));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_get_expr_res5() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_expr_res(Expression::Mul(Value::Num(5), Value::Num(3)));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_get_expr_res6() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_expr_res(Expression::Add(Value::Num(5), Value::Num(3)));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_add_children2() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Constant(Value::Num(5)));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }

    #[test]
    fn test_add_children3() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Add(Value::Num(5), Value::Num(3)));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }

    #[test]

    fn test_add_children4() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Sub(Value::Num(5), Value::Num(3)));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_add_children5() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Div(Value::Num(5), Value::Num(2)));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_add_children6() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Mul(Value::Num(5), Value::Num(3)));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_add_children7() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Avg(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_add_children8() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Max(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_add_children9() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Min(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_add_children10() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Stdev(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_add_children11() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Sum(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_add_children12() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Stringof("Hello".to_string()));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_add_children13() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Sleep(Value::Num(1)));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    #[should_panic]
    fn test_add_children14() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Disable);
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    #[should_panic]
    fn test_add_children15() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::Enable);
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    #[should_panic]
    fn test_add_children16() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::ScrollDown);
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    #[should_panic]
    fn test_add_children17() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::ScrollUp);
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    #[should_panic]
    fn test_add_children18() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::ScrollRight);
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    #[should_panic]
    fn test_add_children19() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.add_children(Cell { row: 1, col: 1 }, Expression::ScrollLeft);
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }

    #[test]
    fn test_get_expr_res7() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_expr_res(Expression::Avg (Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }

    #[test]
    fn test_get_expr_res8() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_expr_res(Expression::Max (Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_get_expr_res9() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_expr_res(Expression::Min (Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_get_expr_res10() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_expr_res(Expression::Stdev (Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_get_expr_res11() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_expr_res(Expression::Sum (Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    #[should_panic]
    fn test_get_expr_res12() {
        let ss = SpreadSheet::new(10, 10);
        ss.get_expr_res(Expression::Stringof ("Hello".to_string()));
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }

    #[test]
    fn test_remove_children2() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.remove_children(Cell { row: 1, col: 1 });
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_remove_children3() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Sum(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        ss.remove_children(Cell { row: 1, col: 1 });
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_remove_children4() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Sub(Value::Num(5), Value::Num(3)),
        }));
        ss.remove_children(Cell { row: 1, col: 1 });
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }

    #[test]
    fn test_remove_children5() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Avg(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        ss.remove_children(Cell { row: 1, col: 1 });
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }

    #[test]
    fn test_remove_children6() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Max(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        ss.remove_children(Cell { row: 1, col: 1 });
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_remove_children7() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Min(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        ss.remove_children(Cell { row: 1, col: 1 });
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_remove_children8() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Stdev(Cell { row: 3, col: 3 }, Cell { row: 4, col: 4 }),
        }));
        ss.remove_children(Cell { row: 1, col: 1 });
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_remove_children9() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Stringof("Hello".to_string()),
        }));
        ss.remove_children(Cell { row: 1, col: 1 });
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }

    #[test]
    fn test_remove_children10() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Sleep(Value::Num(1)),
        }));
        ss.remove_children(Cell { row: 1, col: 1 });
        assert_eq!(ss.val[0], CellVal::IntC(0));
    }
    #[test]
    fn test_check_cycle_big() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Add(Value::Ref(Cell { row: 1, col: 1 }), Value::Num(3)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 3, col: 3 },
            expression: Expression::Add(Value::Ref(Cell { row: 2, col: 2 }), Value::Num(3)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Add(Value::Ref(Cell { row: 3, col: 3 }), Value::Num(3)),
        }));
        assert_eq!(ss.val[22], CellVal::IntC(8));
        assert_eq!(ss.valid[22], 0);
    }

    #[test]
    fn test_get_min3() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Num(5)),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 2, col: 2 },
            expression: Expression::Min(Cell { row: 1, col: 1 }, Cell { row: 1, col: 2 }),
        }));
        ss.call_formula_api(Some(Formula {
            inp_cell: Cell { row: 3, col: 3 },
            expression: Expression::Min(Cell { row: 1, col: 1 }, Cell { row: 2, col: 2 }),
        }));
        ss.get_min(Cell { row: 1, col: 1 }, Cell { row: 3, col: 3 });
        assert_eq!(ss.val[22], CellVal::IntC(0));
    }
}