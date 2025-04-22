use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    pub row: u16,
    pub col: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    pub tl: Cell,
    pub br: Cell,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Num(i32),
    Ref(Cell),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Add(Value, Value),
    Constant(Value),
    Sub(Value, Value),
    Mul(Value, Value),
    Div(Value, Value),
    Min(Cell, Cell),
    Max(Cell, Cell),
    Avg(Cell, Cell),
    Sum(Cell, Cell),
    Stdev(Cell, Cell),
    Sleep(Value),
    Quit,
    Enable,
    Disable,
    ScrollUp,
    ScrollDown,
    ScrollRight,
    ScrollLeft,
    ScrollTo(Cell),
    Stringof(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Formula {
    pub inp_cell: Cell,
    pub expression: Expression,
}

pub enum SpreadSheetError {
    InvalidInput,
    Cycle,
    Valid,
    Quit,
    Enable,
    Disable,
}
