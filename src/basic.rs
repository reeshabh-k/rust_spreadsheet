use std::hash::Hash;



/// Represents a cell in the spreadsheet identified by a row and column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    pub row: u16,
    pub col: u16,
}

/// Represents a range of cells in the spreadsheet, defined by a top-left and bottom-right cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    pub tl: Cell,
    pub br: Cell,
}

/// Represents a value that can either be a number or a reference to a cell.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Num(i32),
    Ref(Cell),
}
/// Represents an expression used in a formula.
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    /// Represents an addition operation between two values.
    Add(Value, Value),
    /// Represents a constant value.
    Constant(Value),
    /// Represents a subtraction operation between two values.
    Sub(Value, Value),
    /// Represents a multiplication operation between two values.
    Mul(Value, Value),
    /// Represents a division operation between two values.
    Div(Value, Value),
    /// Represents the minimum value between two cells.
    Min(Cell, Cell),
    /// Represents the maximum value between two cells.
    Max(Cell, Cell),
    /// Represents the average value between two cells.
    Avg(Cell, Cell),
    /// Represents the sum of values between two cells.
    Sum(Cell, Cell),
    /// Represents the standard deviation of values between two cells.
    Stdev(Cell, Cell),
    /// Represents a sleep operation with a given value.
    Sleep(Value),
    /// Represents the quit operation.
    Quit,
    /// Represents enabling a feature or operation.
    Enable,
    /// Represents disabling a feature or operation.
    Disable,
    /// Represents scrolling up in the spreadsheet.
    ScrollUp,
    /// Represents scrolling down in the spreadsheet.
    ScrollDown,
    /// Represents scrolling right in the spreadsheet.
    ScrollRight,
    /// Represents scrolling left in the spreadsheet.
    ScrollLeft,
    /// Represents scrolling to a specific cell.
    ScrollTo(Cell),
}

/// Represents a formula in the spreadsheet, including the cell it's assigned to and the expression used.
#[derive(Debug, PartialEq, Clone)]
pub struct Formula {
    /// The input cell associated with the formula.
    pub inp_cell: Cell,
    /// The expression that defines the formula.
    pub expression: Expression,
}

/// Represents the possible errors that can occur in the spreadsheet system.
#[derive(PartialEq, Debug, Clone)]
pub enum SpreadSheetError {
    /// Error due to invalid input.
    InvalidInput,
    /// Error due to a cyclic reference in formulas.
    Cycle,
    /// No error, operation was valid.
    Valid,
    /// Error indicating that the quit operation was called.
    Quit,
    /// Error indicating that the enable operation was called.
    Enable,
    /// Error indicating that the disable operation was called.
    Disable,
}
