use arrayvec::ArrayVec;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::BufRead;

use crate::basic::{Cell, Expression, Formula, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Col(pub ArrayVec<u8, 3>);

fn parse_row(row_str: &str) -> Option<u32> {
    let row_num = row_str.parse::<u32>().ok()?;
    if (1..=999).contains(&row_num) {
        Some(row_num)
    } else {
        None
    }
}

fn parse_int(int_str: &str) -> Option<i32> {
    int_str.parse::<i32>().ok()
}

#[allow(dead_code)]
impl Col {
    fn from_str(col_str: &str) -> Option<Col> {
        if col_str.len() > 3 {
            None
        } else {
            for bt in col_str.as_bytes() {
                if !(&b'A'..=&b'Z').contains(&bt) {
                    return None;
                }
            }
            let vec: ArrayVec<u8, 3> = ArrayVec::from_iter(col_str.bytes());
            Some(Col(vec))
        }
    }

    pub fn from_num(mut num: u32) -> Option<Col> {
        if num == 0 || num > 18278 {
            None
        } else {
            let mut col_out: ArrayVec<u8, 3> = ArrayVec::new();

            for i in (0..=2).rev() {
                let f: u32 = 26_u32.pow(i);
                let mut a = num / f;

                if a > 0 || i == 0 || a > 26 {
                    if i > 0 && (num - f * a) <= ((f / 26) + (f / 26 / 26) - 1) {
                        a -= 1
                    }
                    if a == 0 {
                        continue;
                    }

                    let letter = b'A' + (a - 1) as u8;
                    col_out.push(letter);
                    num -= f * a;
                }
            }
            Some(Col(col_out))
        }
    }

    fn num_of_col(&self) -> u32 {
        let mut val: u32 = 0;
        let Col(vec): &Col = self;
        for &bt in vec.iter() {
            val *= 26;
            val += ((bt - b'A') as u32) + 1;
        }
        val
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap()
    }
}

// static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"...").unwrap());
// RE.is_match(haystack)

fn parse_cell(cell_str: &str) -> Option<Cell> {
    let cell_re = Lazy::new(|| Regex::new(r"(?P<col>[A-Z]+)(?P<row>[0-9]+)").unwrap());
    let caps = cell_re.captures(cell_str)?;
    let col_str = &caps["col"];
    let row_str = &caps["row"];
    let temp = Col::from_str(col_str)?.num_of_col();
    Some(Cell {
        col: temp as u16,
        row: parse_row(row_str)? as u16,
    })
}

fn parse_val(val_str: &str) -> Option<Value> {
    if let Some(cell_out) = parse_cell(val_str) {
        return Some(Value::Ref(cell_out));
    }
    if let Some(val_int) = parse_int(val_str) {
        return Some(Value::Num(val_int));
    }
    None
}

// const CELL_RE: &str = r"[A-Z]+[0-9]+";
// const VALUE_RE: &str = r"\d+|[A-Z]+[0-9]+";
// const NUM_RE: &str = r"-?\d+";

// const BINARY_OP_RE: &str = r"(?P<cell>[A-Z]+[0-9]+)";

pub fn get_formula<R: BufRead>(reader: &mut R) -> Option<Formula> {
    let mut line = String::new();
    let _bytes_read = reader.read_line(&mut line);

    let binary_op_re = Lazy::new(|| {
        Regex::new(r"^(?P<cell>[A-Z]+[0-9]+)\s*=\s*(?P<val1>-?\d+|[A-Z]+[0-9]+)\s*(?P<op>[*/+-])\s*(?P<val2>-?\d+|[A-Z]+[0-9]+)\s*$").unwrap()
    });
    let range_op_re = Lazy::new(|| {
        Regex::new(r"^(?P<cell>[A-Z]+[0-9]+)\s*=\s*(?P<op>MAX|MIN|STDEV|AVG|SUM)\s*['(']\s*(?P<cell1>[A-Z]+[0-9]+)\s*:\s*(?P<cell2>[A-Z]+[0-9]+)\s*[')']\s*$").unwrap()
    });
    let sleep_op_re = Lazy::new(|| {
        Regex::new(r"^(?P<cell>[A-Z]+[0-9]+)\s*=\s*SLEEP\s*['(']\s*(?P<val>-?\d+|[A-Z]+[0-9]+)\s*[')']\s*$").unwrap()
    });
    let constant_op_re = Lazy::new(|| {
        Regex::new(r"^(?P<cell>[A-Z]+[0-9]+)\s*=\s*(?P<val>-?\d+|[A-Z]+[0-9]+)\s*$").unwrap()
    });
    let commands_re =
        Lazy::new(|| Regex::new(r"^\s*(?P<command>q|enable_output|disable_output)\s*$").unwrap());
    let scroll_re = Lazy::new(|| Regex::new(r"^\s*(?P<scroll>w|a|s|d)\s*$").unwrap());
    let scroll_to_re =
        Lazy::new(|| Regex::new(r"^\s*scroll_to\s*(?P<cell>[A-Z]+[0-9]+)\s*$").unwrap());

    let string_re = Lazy::new(|| {
        Regex::new("^(?P<cell>[A-Z]+[0-9]+)\\s*=\\s*\"(?P<str>[A-Za-z0-9 ]+)\"\\s*$").unwrap()
    });

    if let Some(caps) = string_re.captures(&line) {
        let cell = parse_cell(&caps["cell"])?;
        let cell_str = &caps["str"];

        let form = Formula {
            inp_cell: cell,
            expression: Expression::Stringof(cell_str.to_string()),
        };
        return Some(form);
    }

    if let Some(caps) = constant_op_re.captures(&line) {
        let cell = parse_cell(&caps["cell"])?;
        let val = parse_val(&caps["val"])?;

        let form = Formula {
            inp_cell: cell,
            expression: Expression::Constant(val),
        };

        return Some(form);
    }

    if let Some(caps) = scroll_to_re.captures(&line) {
        let cell = parse_cell(&caps["cell"])?;

        let form = Formula {
            inp_cell: Cell { row: 0, col: 0 },
            expression: Expression::ScrollTo(cell),
        };

        return Some(form);
    }

    if let Some(caps) = binary_op_re.captures(&line) {
        let cell = parse_cell(&caps["cell"])?;
        let val1 = parse_val(&caps["val1"])?;
        let val2 = parse_val(&caps["val2"])?;

        let op = &caps["op"];

        let form = match op {
            "+" => Formula {
                inp_cell: cell,
                expression: Expression::Add(val1, val2),
            },
            "-" => Formula {
                inp_cell: cell,
                expression: Expression::Sub(val1, val2),
            },
            "/" => Formula {
                inp_cell: cell,
                expression: Expression::Div(val1, val2),
            },
            "*" => Formula {
                inp_cell: cell,
                expression: Expression::Mul(val1, val2),
            },
            _ => return None,
        };

        return Some(form);
    }
    if let Some(caps) = range_op_re.captures(&line) {
        let cell = parse_cell(&caps["cell"])?;
        let cell1 = parse_cell(&caps["cell1"])?;
        let cell2 = parse_cell(&caps["cell2"])?;

        let op = &caps["op"];

        let form = match op {
            "MAX" => Formula {
                inp_cell: cell,
                expression: Expression::Max(cell1, cell2),
            },
            "MIN" => Formula {
                inp_cell: cell,
                expression: Expression::Min(cell1, cell2),
            },
            "AVG" => Formula {
                inp_cell: cell,
                expression: Expression::Avg(cell1, cell2),
            },
            "STDEV" => Formula {
                inp_cell: cell,
                expression: Expression::Stdev(cell1, cell2),
            },
            "SUM" => Formula {
                inp_cell: cell,
                expression: Expression::Sum(cell1, cell2),
            },
            _ => return None,
        };

        return Some(form);
    }
    if let Some(caps) = sleep_op_re.captures(&line) {
        let cell = parse_cell(&caps["cell"])?;
        let val = parse_val(&caps["val"])?;

        let form = Formula {
            inp_cell: cell,
            expression: Expression::Sleep(val),
        };

        return Some(form);
    }
    if let Some(caps) = commands_re.captures(&line) {
        let command = &caps["command"];

        match command {
            "q" => {
                return Some(Formula {
                    inp_cell: Cell { row: 0, col: 0 },
                    expression: Expression::Quit,
                });
            }
            "enable_output" => {
                return Some(Formula {
                    inp_cell: Cell { row: 0, col: 0 },
                    expression: Expression::Enable,
                });
            }
            "disable_output" => {
                return Some(Formula {
                    inp_cell: Cell { row: 0, col: 0 },
                    expression: Expression::Disable,
                });
            }
            _ => panic!("Invalid Sheet Command!"),
        }
    }
    if let Some(caps) = scroll_re.captures(&line) {
        let scroll = &caps["scroll"];

        match scroll {
            "w" => {
                return Some(Formula {
                    inp_cell: Cell { row: 0, col: 0 },
                    expression: Expression::ScrollUp,
                });
            }
            "a" => {
                return Some(Formula {
                    inp_cell: Cell { row: 0, col: 0 },
                    expression: Expression::ScrollLeft,
                });
            }
            "s" => {
                return Some(Formula {
                    inp_cell: Cell { row: 0, col: 0 },
                    expression: Expression::ScrollDown,
                });
            }
            "d" => {
                return Some(Formula {
                    inp_cell: Cell { row: 0, col: 0 },
                    expression: Expression::ScrollRight,
                });
            }
            _ => panic!("Invalid Scroll Command!"),
        }
    }
    None
}


#[cfg(test)]
mod formula_tests {
    use super::*;
    use std::io::Cursor;

    fn test_binary_op(inp_cell: &str, op: &str, val1: &str, val2: &str, form: Formula) {
        let input = format!("{inp_cell}={val1}{op}{val2}");

        let mut buf_inp = Cursor::new(input);
        let form_out = get_formula(&mut buf_inp).expect("Incorrect Input");

        assert_eq!(form_out, form);
    }

    fn test_range_op(inp_cell: &str, op: &str, cell1: &str, cell2: &str, form: Formula) {
        let input = format!("{inp_cell}={op}({cell1}:{cell2})");

        let mut buf_inp = Cursor::new(input);
        let form_out = get_formula(&mut buf_inp).expect("Incorrect Input");

        assert_eq!(form_out, form);
    }
    fn test_sleep_op(inp_cell: &str, val: &str, form: Formula) {
        let input = format!("{inp_cell}=SLEEP({val})");

        let mut buf_inp = Cursor::new(input);
        let form_out = get_formula(&mut buf_inp).expect("Incorrect Input");

        assert_eq!(form_out, form);
    }
    fn test_constant_op(inp_cell: &str, val: &str, form: Formula) {
        let input = format!("{inp_cell}={val}");

        let mut buf_inp = Cursor::new(input);
        let form_out = get_formula(&mut buf_inp).expect("Incorrect Input");

        assert_eq!(form_out, form);
    }
    fn test_command_op(command: &str, form: Formula) {
        let input = format!("{command}");

        let mut buf_inp = Cursor::new(input);
        let form_out = get_formula(&mut buf_inp).expect("Incorrect Input");

        assert_eq!(form_out, form);
    }
    fn test_scroll_op(scroll: &str, form: Formula) {
        let input = format!("{scroll}");

        let mut buf_inp = Cursor::new(input);
        let form_out = get_formula(&mut buf_inp).expect("Incorrect Input");

        assert_eq!(form_out, form);
    }

    fn test_scroll_to_op(cell: &str, form: Formula) {
        let input = format!("scroll_to {cell}");

        let mut buf_inp = Cursor::new(input);
        let form_out = get_formula(&mut buf_inp).expect("Incorrect Input");

        assert_eq!(form_out, form);
    }

    #[test]
    fn scroll_to_op() {
        test_scroll_to_op(
            "A1",
            Formula {
                inp_cell: Cell { row: 0, col: 0 },
                expression: Expression::ScrollTo(Cell { row: 1, col: 1 }),
            },
        );

        test_scroll_to_op(
            "ZZZ898",
            Formula {
                inp_cell: Cell { row: 0, col: 0 },
                expression: Expression::ScrollTo(Cell { row: 898, col: 18278 }),
            },
        );
    }

    #[test]
    fn range_op() {
        test_range_op(
            "A1",
            "MAX",
            "A2",
            "B3",
            Formula {
                inp_cell: Cell { row: 1, col: 1 },
                expression: Expression::Max(
                    Cell { row: 2, col: 1 },
                    Cell { row: 3, col: 2 },
                ),
            },
        );

        test_range_op(
            "ZZZ898",
            "MIN",
            "A2",
            "B3",
            Formula {
                inp_cell: Cell {
                    row: 898,
                    col: 18278,
                },
                expression: Expression::Min(
                    Cell { row: 2, col: 1 },
                    Cell { row: 3, col: 2 },
                ),
            },
        );

        test_range_op(
            "ZZZ898",
            "AVG",
            "A2",
            "B3",
            Formula {
                inp_cell: Cell {
                    row: 898,
                    col: 18278,
                },
                expression: Expression::Avg(
                    Cell { row: 2, col: 1 },
                    Cell { row: 3, col: 2 },
                ),
            },
        );

        test_range_op(
            "ZZZ898",
            "STDEV",
            "A2",
            "B3",
            Formula {
                inp_cell: Cell {
                    row: 898,
                    col: 18278,
                },
                expression: Expression::Stdev(
                    Cell { row: 2, col: 1 },
                    Cell { row: 3, col: 2 },
                ),
            },
        );

        test_range_op(
            "ZZZ898",
            "SUM",
            "A2",
            "B3",
            Formula {
                inp_cell: Cell {
                    row: 898,
                    col: 18278,
                },
                expression: Expression::Sum(
                    Cell { row: 2, col: 1 },
                    Cell { row: 3, col: 2 },
                ),
            },
        );
    }

    #[test]
    fn sleep_op() {
        test_sleep_op(
            "A1",
            "3",
            Formula {
                inp_cell: Cell { row: 1, col: 1 },
                expression: Expression::Sleep(Value::Num(3)),
            },
        );

        test_sleep_op(
            "ZZZ898",
            "-4",
            Formula {
                inp_cell: Cell {
                    row: 898,
                    col: 18278,
                },
                expression: Expression::Sleep(Value::Num(-4)),
            },
        );
    }

    #[test]
    fn constant_op() {
        test_constant_op(
            "A1",
            "3",
            Formula {
                inp_cell: Cell { row: 1, col: 1 },
                expression: Expression::Constant(Value::Num(3)),
            },
        );

        test_constant_op(
            "ZZZ898",
            "-4",
            Formula {
                inp_cell: Cell {
                    row: 898,
                    col: 18278,
                },
                expression: Expression::Constant(Value::Num(-4)),
            },
        );
    }

    #[test]
    fn command_op() {
        test_command_op(
            "q",
            Formula {
                inp_cell: Cell { row: 0, col: 0 },
                expression: Expression::Quit,
            },
        );

        test_command_op(
            "enable_output",
            Formula {
                inp_cell: Cell {
                    row: 0,
                    col: 0,
                },
                expression: Expression::Enable,
            },
        );

        test_command_op(
            "disable_output",
            Formula {
                inp_cell: Cell {
                    row: 0,
                    col: 0,
                },
                expression: Expression::Disable,
            },
        );
    }
    
    #[test]
    fn scroll_op() {
        test_scroll_op(
            "w",
            Formula {
                inp_cell: Cell { row: 0, col: 0 },
                expression: Expression::ScrollUp,
            },
        );

        test_scroll_op(
            "a",
            Formula {
                inp_cell: Cell {
                    row: 0,
                    col: 0,
                },
                expression: Expression::ScrollLeft,
            },
        );

        test_scroll_op(
            "s",
            Formula {
                inp_cell: Cell {
                    row: 0,
                    col: 0,
                },
                expression: Expression::ScrollDown,
            },
        );

        test_scroll_op(
            "d",
            Formula {
                inp_cell: Cell {
                    row: 0,
                    col: 0,
                },
                expression: Expression::ScrollRight,
            },
        );
    }

    #[test]
    #[should_panic]
    fn invalid_scroll() {
        test_scroll_op(
            "x",
            Formula {
                inp_cell: Cell {
                    row: 0,
                    col: 0,
                },
                expression: Expression::ScrollRight,
            },
        );
    }

    #[test]
    fn binary_op() {
        test_binary_op(
            "A1",
            "+",
            "-4",
            "3",
            Formula {
                inp_cell: Cell { row: 1, col: 1 },
                expression: Expression::Add(Value::Num(-4), Value::Num(3)),
            },
        );

        test_binary_op(
            "ZZZ898",
            "*",
            "   -4",
            "        3\n",
            Formula {
                inp_cell: Cell {
                    row: 898,
                    col: 18278,
                },
                expression: Expression::Mul(Value::Num(-4), Value::Num(3)),
            },
        );

        test_binary_op(
            "ZZZ898",
            "/",
            "   -4    ",
            "        3\n",
            Formula {
                inp_cell: Cell {
                    row: 898,
                    col: 18278,
                },
                expression: Expression::Div(Value::Num(-4), Value::Num(3)),
            },
        );

        test_binary_op(
            "ZZZ898",
            "-",
            "   4    ",
            "        3\n",
            Formula {
                inp_cell: Cell {
                    row: 898,
                    col: 18278,
                },
                expression: Expression::Sub(Value::Num(4), Value::Num(3)),
            },
        );

        //Complete more tests
    }
}

#[cfg(test)]
mod col_tests {
    use super::*;

    fn test_col_of_num(col_str: &str, num: u32) {
        let col_exp = Col::from_str(col_str);
        let col_out = Col::from_num(num);

        assert_eq!(col_exp, col_out);
    }

    fn test_num_of_col(col_str: &str, num: u32) {
        let col = Col::from_str(col_str).expect("Column String Out of Range\n");
        let num_exp = col.num_of_col();

        assert_eq!(num_exp, num);
    }

    fn test_num_of_col_of_num(num: u32) {
        let col = Col::from_num(num).expect("Column Number out of Range\n");
        let num_out = col.num_of_col();

        assert_eq!(num, num_out);
    }

    fn test_to_str(col: Col, col_str: &str) {
        let col_out = col.as_str();
        assert_eq!(col_out, col_str);
    }

    #[test]
    fn col_to_str() {
        let col = Col::from_str("A").expect("Invalid Column String");
        test_to_str(col, "A");

        let col = Col::from_str("ZZZ").expect("Invalid Column String");
        test_to_str(col, "ZZZ");

        let col = Col::from_str("AB").expect("Invalid Column String");
        test_to_str(col, "AB");
    }


    #[test]
    fn create_a() {
        let col_str = "A";
        let col_out = Col::from_str(col_str).expect("Invalid String");
        let mut col_exp: ArrayVec<u8, 3> = ArrayVec::<u8, 3>::new();
        col_exp.push(b'A');
        let col_exp = Col(col_exp);

        assert_eq!(col_out, col_exp);
    }

    #[test]
    fn create_gf() {
        let col_str = "GF";
        let col_out = Col::from_str(col_str).expect("Invalid String");
        let mut col_exp: ArrayVec<u8, 3> = ArrayVec::new();
        col_exp.push(b'G');
        col_exp.push(b'F');

        let col_exp = Col(col_exp);

        assert_eq!(col_out, col_exp);
    }

    #[test]
    fn create_zzz() {
        let col_str = "ZZZ";
        let col_out = Col::from_str(col_str).expect("Invalid String");
        let col_exp: ArrayVec<u8, 3> = ArrayVec::from([b'Z'; 3]);
        let col_exp = Col(col_exp);

        assert_eq!(col_out, col_exp);
    }

    #[test]
    #[should_panic]
    fn create_aaaa() {
        let col_str = "AAAA";
        let _col_out = Col::from_str(col_str).expect("Invalid String");
    }

    #[test]
    #[should_panic]
    fn create_lower_a() {
        let col_str = "a";
        let _col_out = Col::from_str(col_str).expect("Invalid String");
    }

    #[test]
    fn num_of_col() {
        test_num_of_col("A", 1);
        test_num_of_col("D", 4);
        test_num_of_col("AB", 28);
        test_num_of_col("MA", 339);
        test_num_of_col("YE", 655);
        test_num_of_col("DEF", 2840);
        test_num_of_col("ZKM", 17875);
        test_num_of_col("ZZZ", 18278);
    }

    

    #[test]
    fn col_of_num() {
        test_col_of_num("A", 1);
        test_col_of_num("D", 4);
        test_col_of_num("AB", 28);
        test_col_of_num("MA", 339);
        test_col_of_num("YE", 655);
        test_col_of_num("JZ", 286);
        test_col_of_num("DEF", 2840);
        test_col_of_num("QZP", 12184);
        test_col_of_num("ZKM", 17875);
        test_col_of_num("ZZZ", 18278);
    }

    #[test]
    fn num_of_col_of_num() {
        for i in 1..=18278 {
            test_num_of_col_of_num(i);
        }
    }
}

#[cfg(test)]
mod row_tests {
    use super::*;
    fn test_row(row_str: &str, row: u32) {
        let row_out = parse_row(row_str).expect("Invalid Row");
        assert_eq!(row_out, row);
    }
    #[test]
    fn row() {
        test_row("1", 1);
        test_row("999", 999);
        test_row("110", 110);
        // test_row("0", 0);
        test_row("100", 100);
        test_row("10", 10);
    }
    #[test]
    #[should_panic]
    fn row_out_of_range() {
        test_row("1000", 1000);
    }
    #[test]
    #[should_panic]
    fn row_zero() {
        test_row("-1", 0);
    }

}


#[cfg(test)]
mod parse_tests {
    use super::*;

    fn test_parse_cell(cell_str: &str, cell: Cell) {
        let cell_out = parse_cell(cell_str).expect("Invalid Cell");
        assert_eq!(cell_out, cell);
    }

    fn test_parse_val(val_str: &str, val: Value) {
        let val_out = parse_val(val_str).expect("Invalid Value");
        assert_eq!(val_out, val);
    }

    #[test]
    fn parse_cell_test() {
        test_parse_cell("A1", Cell { col: 1, row: 1 });
        test_parse_cell("ZZZ898", Cell { col: 18278, row: 898 });
        test_parse_cell("AB10", Cell { col: 28, row: 10 });
        test_parse_cell("MA339", Cell { col: 339, row: 339 });
        test_parse_cell("YE655", Cell { col: 655, row: 655 });
        test_parse_cell("DEF280", Cell { col: 2840, row: 280 });
        test_parse_cell("ZKM175", Cell { col: 17875, row: 175 });
        test_parse_cell("ZZZ188", Cell { col: 18278, row: 188 });
    }

    #[test]
    fn parse_val_test() {
        test_parse_val("A1", Value::Ref(Cell { col: 1, row: 1 }));
        test_parse_val("ZZZ898", Value::Ref(Cell { col: 18278, row: 898 }));
        test_parse_val("-4", Value::Num(-4));
        test_parse_val("3", Value::Num(3));
        test_parse_val("0", Value::Num(0));
        test_parse_val("100", Value::Num(100));
    }
}

