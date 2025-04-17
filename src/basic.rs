#[derive(Debug, PartialEq, Clone)]
pub struct Cell {
    pub row: u16,
    pub col: u16,
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
}

