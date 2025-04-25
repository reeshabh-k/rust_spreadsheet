use crate::input::Col;
use crate::{
    basic::Cell, basic::Expression, basic::Formula, basic::SpreadSheetError, basic::Value,
};

use std::thread;
use std::time::Duration;

use std::{collections::HashMap, collections::HashSet};


/// A structure that holds information about a cell, such as its children (dependent cells).
/// This helps in managing dependencies between cells when formulas are used.
#[derive(Clone, Debug)]
struct CellData {
    children: HashSet<Cell>,
}

/// A structure representing the entire spreadsheet, holding the data for all cells.
/// It supports expressions, formulas, and dependency tracking between cells.
pub struct SpreadSheet {
    row_pointer: usize,
    col_pointer: usize,
    row: usize,
    col: usize,
    spreadsheet: Vec<CellData>,
    val: Vec<i32>,
    valid: Vec<u8>,
    exprs: HashMap<Cell, Expression>,
    constant_mode: u32,
}


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
            val: vec![0; (row + 1) * (col + 1)],
            valid: vec![0; (row + 1) * (col + 1)],
            exprs: HashMap::new(),
            constant_mode: 1,
        }
    }

    /// Updates the value of a cell based on its expression.
    ///
    /// If the cell has an expression, it will evaluate the expression and update
    /// the value of the cell accordingly.
    ///
    /// # Parameters
    /// - `cell`: The cell to update.
    fn update_cell(&mut self, cell: Cell) {
        let row = cell.row as usize;
        let col = cell.col as usize;

        let cell_loc = self.col * row + col;
        // let _cell_data = &self.spreadsheet[cell_loc];

        let expr = if self.exprs.contains_key(&cell) {
            self.exprs.get(&cell).expect("Weird!")
        } else {
            return;
        };

        let eval_expr = self.get_expr_res(expr.clone());

        match eval_expr {
            None => self.valid[cell_loc] = 1,
            Some(i) => {
                self.valid[cell_loc] = 0;
                self.val[cell_loc] = i
            }
        }
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
                } else {
                    Some(self.val[cell_point])
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
                // if self.spreadsheet[self.get_pointer(&Cell {row : i, col : j})].valid == 1 {
                //     return None;
                // }
                sum += self.val[self.get_pointer(&Cell { row: i, col: j })];
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
                sum += self.val[self.get_pointer(&Cell { row: i, col: j })];
                cnt += 1;
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
        max = self.val[self.get_pointer(&Cell {
            row: c1.row,
            col: c1.col,
        })];
        // min = self.spreadsheet[self.get_pointer(&Cell {row : c1.row, col : c1.col})].val;
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell { row: i, col: j })] == 1 {
                    return None;
                }
                let val = self.val[self.get_pointer(&Cell { row: i, col: j })];
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
        min = self.val[self.get_pointer(&Cell {
            row: c1.row,
            col: c1.col,
        })];
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell { row: i, col: j })] == 1 {
                    return None;
                }
                let val = self.val[self.get_pointer(&Cell { row: i, col: j })];
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
        let mut temp;
        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                if self.valid[self.get_pointer(&Cell { row: i, col: j })] == 1 {
                    return None;
                }
                mean += self.val[self.get_pointer(&Cell { row: i, col: j })];
                cnt += 1;
            }
        }
        mean /= cnt;

        for i in c1.row..=c2.row {
            for j in c1.col..=c2.col {
                temp = self.val[self.get_pointer(&Cell { row: i, col: j })];
                variance += ((temp - mean) * (temp - mean)) as f64;
            }
        }
        variance /= cnt as f64;
        let stddev = variance.sqrt();
        Some(stddev.round() as i32)
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
    fn get_expr_res(&self, expr: Expression) -> Option<i32> {
        match expr {
            Expression::Add(v1, v2) => {
                Some(self.extract_value_num(v1)? + self.extract_value_num(v2)?)
            }
            Expression::Mul(v1, v2) => {
                Some(self.extract_value_num(v1)? * self.extract_value_num(v2)?)
            }
            Expression::Div(v1, v2) => {
                let denom = self.extract_value_num(v2)?;
                if denom == 0 {
                    None
                } else {
                    Some(self.extract_value_num(v1)? / denom)
                }
            }
            Expression::Sub(v1, v2) => {
                Some(self.extract_value_num(v1)? - self.extract_value_num(v2)?)
            }
            Expression::Constant(v) => self.extract_value_num(v),
            Expression::Sleep(v) => {
                let sleep_time = self.extract_value_num(v)?;
                thread::sleep(Duration::from_secs(sleep_time as u64));
                Some(sleep_time)
            }

            Expression::Avg(c1, c2) => {
                // let (_, _, sum_ele, _) = self.recursive_row_split(Range {tl: c1, br:c2})?;
                // Some(sum_ele / ((c2.row as i32  - c1.row as i32+ 1) * (c2.col as i32  - c1.col as i32 + 1)))
                self.get_avg(c1, c2)
            }
            Expression::Max(c1, c2) => {
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

            _ => panic!("Expression should not be called!"),
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
    fn get_pointer(&self, inp_cell: &Cell) -> usize {
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

            _ => panic!("Unimplemented add_children!"),
        }
    }

    
    /// Handles formula calls, processes different expressions, updates spreadsheet state, and handles cycles.
    ///
    /// # Arguments
    /// * `form`: An `Option<Formula>`, where `Some(valid_form)` contains a valid formula and `None` indicates invalid input.
    ///
    /// # Returns
    /// * `SpreadSheetError`: A type indicating the outcome of the operation, such as `Valid`, `InvalidInput`, or `Cycle`.
    ///
    /// This function processes various spreadsheet expressions, handles scrolling, and evaluates cell expressions. 
    /// It also ensures that cyclic dependencies are checked to prevent infinite loops.
    pub fn call_formula(&mut self, form: Option<Formula>) -> SpreadSheetError {
        // println!("Constant Mode: {}", self.constant_mode);
        let form = match form {
            None => return SpreadSheetError::InvalidInput,
            Some(valid_form) => valid_form,
        };
        let cell_pointer = self.col * form.inp_cell.row as usize + form.inp_cell.col as usize;
        match form.expression {
            Expression::Quit => return SpreadSheetError::Quit,
            Expression::Disable => return SpreadSheetError::Disable,
            Expression::Enable => return SpreadSheetError::Enable,
            Expression::ScrollDown => {
                self.row_pointer = (self.row_pointer + 10).min(self.row - 9);
                return SpreadSheetError::Valid;
            }
            Expression::ScrollUp => {
                self.row_pointer = self.row_pointer.saturating_sub(10).max(1);
                return SpreadSheetError::Valid;
            }
            Expression::ScrollRight => {
                self.col_pointer = (self.col_pointer + 10).min(self.col - 9);
                return SpreadSheetError::Valid;
            }
            Expression::ScrollLeft => {
                self.col_pointer = self.col_pointer.saturating_sub(10).max(1);
                return SpreadSheetError::Valid;
            }
            Expression::ScrollTo(c) => {
                self.col_pointer = c.col as usize;
                self.row_pointer = c.row as usize;
                return SpreadSheetError::Valid;
            }
            Expression::Avg(c1, c2)
            | Expression::Max(c1, c2)
            | Expression::Min(c1, c2)
            | Expression::Sum(c1, c2)
            | Expression::Stdev(c1, c2) => {
                if c1.row > c2.row || c1.col > c2.col {
                    //comment
                    return SpreadSheetError::InvalidInput;
                }
                self.constant_mode = 0;
            }
            Expression::Constant(Value::Num(i)) => {
                if self.constant_mode == 1 {
                    self.val[cell_pointer] = i;
                    return SpreadSheetError::Valid;
                }
            }

            _ => {
                self.constant_mode = 0;
            }
        }
        if self.check_cycle(form.clone()) {
            return SpreadSheetError::Cycle;
        }

        self.remove_children(form.inp_cell);

        self.exprs.insert(form.inp_cell, form.expression.clone());

        self.add_children(form.inp_cell, form.expression);

        // match form.expression {
        //     Expression::Add(Value::Num(_), Value::Num(_))
        //     | Expression::Mul(Value::Num(_), Value::Num(_))
        //     | Expression::Div(Value::Num(_), Value::Num(_))
        //     | Expression::Sub(Value::Num(_), Value::Num(_))
        //     | Expression::Sleep(Value::Num(_))
        //     | Expression::Constant(Value::Num(_)) => {
        //         if self.exprs.contains_key(&form.inp_cell) {
        //             self.exprs.remove(&form.inp_cell);
        //         }
        //         self.val[cell_pointer] = self
        //             .get_expr_res(form.expression.clone())
        //             .expect("Invalid Expression");
        //     }

        //     _ => {
        //         self.exprs.insert(form.inp_cell, form.expression.clone());
        //         self.add_children(form.inp_cell, form.expression);
        //     }
        // }

        self.update_children(form.inp_cell);

        SpreadSheetError::Valid
    }

    /// Updates all children cells for a given cell `inp_cell` based on its dependencies and expression results.
    ///
    /// # Arguments
    /// * `inp_cell`: The `Cell` whose children need to be updated.
    ///
    /// # Returns
    /// This function does not return anything. It mutates the spreadsheet state.
    fn update_children(&mut self, inp_cell: Cell) {
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

        for (i, _) in sorted_vec.iter() {
            self.update_cell(*i);
        }
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
                    print!("{:<10}", self.val[cell_pointer + j as usize]);
                }
            }
            println!();
        }
    }
}


#[cfg(test)]

mod spreadsheet_tests {
    use std::cell;

    use super::*;
    use crate::basic::Expression;
    use crate::basic::Value;

    #[test]
    fn test_add_children() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        assert_eq!(ss.exprs.len(), 1);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 1);
    }

    #[test]
    fn test_remove_children() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        assert_eq!(ss.exprs.len(), 1);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell{row : 2, col: 2})].children.len(), 1);
        ss.remove_children(cell);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell{row : 2, col: 2})].children.len(), 0);
    }

    #[test]
    fn test_update_cell() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let ptr = ss.get_pointer(&cell);
        ss.val[ptr] = 5;
        ss.update_cell(cell);
        assert_eq!(ss.val[ss.get_pointer(&cell)], 5);
    }

    #[test]
    fn test_get_pointer() {
        let ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        assert_eq!(ss.get_pointer(&cell), 11);
    }

    #[test]
    fn test_get_expr_res() {
        let mut ss = SpreadSheet::new(10, 10);
        let _cell = Cell { row: 1, col: 1 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        let ptr = ss.get_pointer(&Cell { row: 2, col: 2 });
        ss.val[ptr] = 3;
        assert_eq!(ss.get_expr_res(expr), Some(8));
    }

    #[test]
    fn test_check_cycle() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        let ptr = ss.get_pointer(&Cell { row: 2, col: 2 });
        ss.val[ptr] = 3;
        assert_eq!(ss.check_cycle(Formula { inp_cell: cell, expression: expr }), false);
    }

    #[test]
    fn test_update_children() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        ss.call_formula(Some(Formula { inp_cell: cell, expression: Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 })) }));
        let ptr = ss.get_pointer(&Cell { row: 2, col: 2 });
        ss.val[ptr] = 3;
        ss.update_children(cell);
        assert_eq!(ss.val[ss.get_pointer(&cell)], 8);
    }

    #[test]
    fn test_print_sheet() {
        let mut ss = SpreadSheet::new(10, 10);
        let ptr = ss.get_pointer(&Cell { row: 1, col: 1 });
        ss.val[ptr] = 5;
        ss.print_sheet();
    }

    #[test]
    fn test_call_formula() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        let ptr = ss.get_pointer(&Cell { row: 2, col: 2 });
        ss.val[ptr] = 3;
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Valid);
    }

    #[test]
    fn test_get_avg() {
        let mut ss = SpreadSheet::new(10, 10);
        let mut ptr = ss.get_pointer(&Cell { row: 1, col: 1 });
        ss.val[ptr] = 5;
        ptr = ss.get_pointer(&Cell { row: 1, col: 2 });
        ss.val[ptr] = 3;
        assert_eq!(ss.get_avg(Cell { row: 1, col: 1 }, Cell { row: 1, col: 2 }), Some(4));
    }

    #[test]
    fn test_get_max() {
        let mut ss = SpreadSheet::new(10, 10);
        let mut ptr = ss.get_pointer(&Cell { row: 1, col: 1 });
        ss.val[ptr] = 5;
        ptr = ss.get_pointer(&Cell { row: 1, col: 2 });
        ss.val[ptr] = 3;
        assert_eq!(ss.get_max(Cell { row: 1, col: 1 }, Cell { row: 1, col: 2 }), Some(5));
    }

    #[test]
    fn test_get_min() {
        let mut ss = SpreadSheet::new(10, 10);
        let mut ptr = ss.get_pointer(&Cell { row: 1, col: 1 });
        ss.val[ptr] = 5;
        ptr = ss.get_pointer(&Cell { row: 1, col: 2 });
        ss.val[ptr] = 3;
        assert_eq!(ss.get_min(Cell { row: 1, col: 1 }, Cell { row: 1, col: 2 }), Some(3));
    }

    #[test]
    fn test_get_stddev() {
        let mut ss = SpreadSheet::new(10, 10);
        let mut ptr = ss.get_pointer(&Cell { row: 1, col: 1 });
        ss.val[ptr] = 5;
        ptr = ss.get_pointer(&Cell { row: 1, col: 2 });
        ss.val[ptr] = 3;
        assert_eq!(ss.get_stddev(Cell { row: 1, col: 1 }, Cell { row: 1, col: 2 }), Some(1));
    }

    #[test]
    fn test_get_sum() {
        let mut ss = SpreadSheet::new(10, 10);
        let mut ptr = ss.get_pointer(&Cell { row: 1, col: 1 });
        ss.val[ptr] = 5;
        ptr = ss.get_pointer(&Cell { row: 1, col: 2 });
        ss.val[ptr] = 3;
        assert_eq!(ss.get_sum(Cell { row: 1, col: 1 }, Cell { row: 1, col: 2 }), Some(8));
    }

    #[test]
    fn test_extract_value_num() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let ptr = ss.get_pointer(&cell);
        ss.val[ptr] = 5;
        assert_eq!(ss.extract_value_num(Value::Num(5)), Some(5));
        assert_eq!(ss.extract_value_num(Value::Ref(cell)), Some(5));
    }

    #[test]
    fn test_extract_value_num_invalid() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        ss.call_formula(Some(Formula { inp_cell: cell, expression: Expression::Avg(Cell {row: 1, col:4 }, Cell{row: 2, col: 3}) }));
        assert_eq!(ss.extract_value_num(Value::Ref(Cell { row: 1, col: 1 })), Some(0));
    }

    #[test] 
    fn test_spreadsheet() {
        let mut ss = SpreadSheet::new(10, 10);
        ss.print_sheet();
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Valid);
        ss.print_sheet();
    }
    #[test]
    fn test_spreadsheet_cycle() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell1 = Cell { row: 1, col: 1 };
        let cell2 = Cell { row: 2, col: 2 };
        let expr1 = Expression::Add(Value::Num(5), Value::Ref(cell2));
        let expr2 = Expression::Add(Value::Num(3), Value::Ref(cell1));
        let form1 = Formula { inp_cell: cell1, expression: expr1 };
        let form2 = Formula { inp_cell: cell2, expression: expr2 };
        assert_eq!(ss.call_formula(Some(form1)), SpreadSheetError::Valid);
        assert_eq!(ss.call_formula(Some(form2)), SpreadSheetError::Cycle);
    }
    #[test]
    fn test_spreadsheet_invalid_input() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Valid);
        assert_eq!(ss.call_formula(None), SpreadSheetError::InvalidInput);
    }
    #[test]
    fn test_spreadsheet_quit() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Quit;
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Quit);
    }
    #[test]
    fn test_spreadsheet_enable() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Enable;
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Enable);
    }
    #[test]
    fn test_spreadsheet_disable() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Disable;
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Disable);
    }
    #[test]
    fn test_spreadsheet_scroll_down() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::ScrollDown;
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Valid);
    }
    #[test]
    fn test_spreadsheet_scroll_up() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::ScrollUp;
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Valid);
    }
    #[test]
    fn test_spreadsheet_scroll_right() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::ScrollRight;
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Valid);
    }
    #[test]
    fn test_spreadsheet_scroll_left() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::ScrollLeft;
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Valid);
    }
    #[test]
    fn test_spreadsheet_scroll_to() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::ScrollTo(Cell { row: 2, col: 2 });
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Valid);
    }
    #[test]
    fn test_spreadsheet_invalid_range() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Avg(Cell { row: 2, col: 2 }, Cell { row: 1, col: 1 });
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::InvalidInput);
    }
    #[test]
    fn test_spreadsheet_invalid_expression() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Valid);
    }
    #[test]
    fn test_spreadsheet_invalid_cell() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Valid);
    }
    #[test]
    fn test_spreadsheet_invalid_formula() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Valid);
    }
    #[test]
    fn test_spreadsheet_invalid_formula2() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        let form = Formula { inp_cell: cell, expression: expr };
        assert_eq!(ss.call_formula(Some(form)), SpreadSheetError::Valid);
    }

    #[test]
    fn test_add_children2() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Sub (Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        assert_eq!(ss.exprs.len(), 1);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 1);
    }
    #[test]
    fn test_add_children3() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Div (Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        assert_eq!(ss.exprs.len(), 1);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 1);
    }
    #[test]
    fn test_add_children4() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Mul (Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        assert_eq!(ss.exprs.len(), 1);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 1);
    }
    #[test]
    fn test_add_children5() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Sleep (Value::Num(1));
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        assert_eq!(ss.exprs.len(), 1);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 0);
    }
    #[test]
    fn test_add_children6() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Constant (Value::Num(5));
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        assert_eq!(ss.exprs.len(), 0);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 0);
    }
    #[test]
    fn test_add_children7() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Avg (Cell { row: 2, col: 2 }, Cell { row: 3, col: 3 });
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        assert_eq!(ss.exprs.len(), 1);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 1);
    }
    #[test]
    fn test_add_children8() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Max (Cell { row: 2, col: 2 }, Cell { row: 3, col: 3 });
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        assert_eq!(ss.exprs.len(), 1);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 1);
    }
    #[test]
    fn test_add_children9() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Min (Cell { row: 2, col: 2 }, Cell { row: 3, col: 3 });
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        assert_eq!(ss.exprs.len(), 1);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 1);
    }
    #[test]
    fn test_add_children10() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Sum (Cell { row: 2, col: 2 }, Cell { row: 3, col: 3 });
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        assert_eq!(ss.exprs.len(), 1);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 1);
    }
    #[test]
    fn test_add_children11() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Stdev (Cell { row: 2, col: 2 }, Cell { row: 3, col: 3 });
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        assert_eq!(ss.exprs.len(), 1);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 1);
    }
    #[test]
    fn test_add_children12() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let expr = Expression::Stdev (Cell { row: 2, col: 2 }, Cell { row: 3, col: 3 });
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        assert_eq!(ss.exprs.len(), 1);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 1);
    }

    #[test]
    fn test_remove_children2() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let cell2 = Cell { row: 2, col: 2 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        let expr2 = Expression::Avg(Cell { row: 2, col: 2 }, Cell { row: 3, col: 3 });
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        ss.call_formula(Some(Formula{ inp_cell: cell2, expression: expr2.clone() }));
        assert_eq!(ss.exprs.len(), 1);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 1);
        ss.remove_children(cell);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell{row : 2, col: 2})].children.len(), 0);
    }

    #[test]
    fn test_check_cycle2() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let cell2 = Cell { row: 2, col: 2 };
        let cell3 = Cell { row: 3, col: 3 };
        let cell4 = Cell { row: 4, col: 4 };
        let cell5 = Cell { row: 5, col: 5 };
        let cell6 = Cell { row: 6, col: 6 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        let expr2 = Expression::Avg(Cell { row: 2, col: 2 }, Cell { row: 3, col: 3 });
        let expr3 = Expression::Max(Cell { row: 2, col: 2 }, Cell { row: 3, col: 3 });
        let expr4 = Expression::Min(Cell { row: 2, col: 2 }, Cell { row: 3, col: 3 });
        let expr5 = Expression::Stdev(Cell { row: 2, col: 2 }, Cell { row: 3, col: 3 });
        let expr6 = Expression::Sum(Cell { row: 2, col: 2 }, Cell { row: 3, col: 3 });
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        ss.call_formula(Some(Formula{ inp_cell: cell2, expression: expr2.clone() }));
        ss.call_formula(Some(Formula{ inp_cell: cell3, expression: expr3.clone() }));
        ss.call_formula(Some(Formula{ inp_cell: cell4, expression: expr4.clone() }));
        ss.call_formula(Some(Formula{ inp_cell: cell5, expression: expr5.clone() }));
        ss.call_formula(Some(Formula{ inp_cell: cell6, expression: expr6.clone() }));
        assert_eq!(ss.exprs.len(), 4);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 4);
        assert_eq!(ss.check_cycle(Formula{ inp_cell: cell4, expression: expr4 }), false);
    }

    #[test]
    fn test_remove_children3() {
        let mut ss = SpreadSheet::new(10, 10);
        let cell = Cell { row: 1, col: 1 };
        let cell2 = Cell { row: 2, col: 2 };
        let expr = Expression::Add(Value::Num(5), Value::Ref(Cell { row: 2, col: 2 }));
        let _expr2 = Expression::Avg(Cell { row: 2, col: 2 }, Cell { row: 3, col: 3 });
        let expr3 = Expression::Sleep(Value::Num(1));
        ss.call_formula(Some(Formula{ inp_cell: cell, expression: expr.clone() }));
        ss.call_formula(Some(Formula{ inp_cell: cell2, expression: expr3.clone() }));
        assert_eq!(ss.exprs.len(), 2);
        // assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell {row:2, col:2})].children.len(), 1);
        ss.remove_children(cell2);
        assert_eq!(ss.spreadsheet[ss.get_pointer(&Cell{row : 2, col: 2})].children.len(), 1);
    }
   
   
}