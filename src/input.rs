struct Cell {
    row: u32,
    col: u32,
}

enum Value {
    Num(i32),
    Ref(Cell),
}

enum Formula {
    Add(Value, Value),
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

fn col_of_num (num: u32) -> 

fn get_formula () -> Formula {

}
