use arrayvec::ArrayVec;
use std::{f32::consts::E, io::{self, BufRead, Error}, thread::sleep};
use regex::Regex;
use once_cell::sync::Lazy;
use std::io::Cursor;
use std::fmt::{self, Display, Formatter};


use crate::basic::{Cell, Formula, Expression, Value};




#[derive(Debug, Clone, PartialEq)]
pub struct Col(pub ArrayVec<u8, 3>);

fn parse_row (row_str: &str) -> Option<u32> {
    let row_num = row_str.parse::<u32>().ok()?;
    if row_num >= 1 && row_num <= 999 {
        Some(row_num)
    } else {
        None
    }
}

fn parse_int (int_str: &str) -> Option<i32> {
    Some(int_str.parse::<i32>().ok()?)
}

    
impl Col {
    fn from_str(col_str: &str) -> Option<Col> {
        if col_str.len() > 3 {
            None
        } else {
            for bt in col_str.as_bytes() {
                if bt < &b'A' || bt > &b'Z' {
                    return None;
                } 
            }
            let vec: ArrayVec<u8, 3> = ArrayVec::from_iter(col_str.bytes());
            Some(Col(vec))
        }
    }

    pub fn from_num (mut num : u32) -> Option<Col> {
        if num == 0 || num > 18278 {
            None
        } else {
            let mut col_out: ArrayVec<u8, 3> = ArrayVec::new();

            for i in (0..=2).rev() {
                let f: u32 = (26 as u32).pow(i);
                let mut a = num / f;

                if a > 0 || i == 0 || a > 26 {

                    if i > 0 && (num - f * a) <= ((f/26) + (f/26/26) - 1) {a -= 1}
                    if a == 0 {continue}

                    let letter = b'A' + (a-1) as u8;
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

fn parse_cell (cell_str: &str) -> Option<Cell> {
    let cell_re = Lazy::new(|| Regex::new(r"(?P<col>[A-Z]+)(?P<row>[0-9]+)").unwrap());
    let caps = cell_re.captures(cell_str)?;

    Some(
        Cell {
            col: Col::from_str(&caps["col"])?.num_of_col() as u16,
            row: parse_row(&caps["row"])? as u16,
        }
    )
}

fn parse_val (val_str: &str) -> Option<Value> {
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

pub fn get_formula<R: BufRead>(reader: & mut R) -> Option<Formula> {
    let mut line = String::new();
    let _bytes_read = reader.read_line(&mut line);

     
    let binary_op_re = Lazy::new(|| Regex::new(r"^(?P<cell>[A-Z]+[0-9]+)\s*=\s*(?P<val1>-?\d+|[A-Z]+[0-9]+)\s*(?P<op>[*/+-])\s*(?P<val2>-?\d+|[A-Z]+[0-9]+)\s*$").unwrap());
    let range_op_re = Lazy::new(|| Regex::new(r"^(?P<cell>[A-Z]+[0-9]+)\s*=\s*(?P<op>MAX|MIN|STDEV|AVG|SUM)\s*['(']\s*(?P<cell1>[A-Z]+[0-9]+)\s*:\s*(?P<cell2>[A-Z]+[0-9]+)\s*[')']\s*$").unwrap());
    let sleep_op_re = Lazy::new(|| Regex::new(r"^(?P<cell>[A-Z]+[0-9]+)\s*=\s*SLEEP\s*['(']\s*(?P<val>-?\d+|[A-Z]+[0-9]+)\s*[')']\s*$").unwrap());
    let constant_op_re = Lazy::new(|| Regex::new(r"^(?P<cell>[A-Z]+[0-9]+)\s*=\s*(?P<val>-?\d+|[A-Z]+[0-9]+)\s*$").unwrap());
    let commands_re = Lazy::new(|| Regex::new(r"^\s*(?P<command>q|enable_output|disable_output)\s*$").unwrap());
    let scroll_re = Lazy::new(|| Regex::new(r"^\s*(?P<scroll>w|a|s|d)\s*$").unwrap());
    let scroll_to_re = Lazy::new(|| Regex::new(r"^\s*scroll_to\s*(?P<cell>[A-Z]+[0-9]+)\s*$").unwrap());

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
            inp_cell: Cell {row : 0, col : 0},
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
                expression: Expression::Add(val1, val2)
            },
            "-" => Formula {
                inp_cell: cell,
                expression: Expression::Sub(val1, val2)
            },
            "/" => Formula {
                inp_cell: cell,
                expression: Expression::Div(val1, val2)
            },
            "*" => Formula {
                inp_cell: cell,
                expression: Expression::Mul(val1, val2)
            },
            _ => return None
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
                expression: Expression::Max(cell1, cell2)
            },
            "MIN" => Formula {
                inp_cell: cell,
                expression: Expression::Min(cell1, cell2)
            },
            "AVG" => Formula {
                inp_cell: cell,
                expression: Expression::Avg(cell1, cell2)
            },
            "STDEV" => Formula {
                inp_cell: cell,
                expression: Expression::Stdev(cell1, cell2)
            },
            "SUM" => Formula {
                inp_cell: cell,
                expression: Expression::Sum(cell1, cell2)
            },
            _ => return None
        };

        return Some(form)
    }
    if let Some(caps) = sleep_op_re.captures(&line) {
        let cell = parse_cell(&caps["cell"])?;
        let val = parse_val(&caps["val"])?;
        
        let form = Formula {
            inp_cell: cell,
            expression: Expression::Sleep(val)
        };

        return Some(form)
    }
    if let Some(caps) = commands_re.captures(&line) {
        let command = &caps["command"];

        match command {
            "q" => return Some(
                Formula {
                    inp_cell: Cell{row: 0, col: 0},
                    expression: Expression::Quit,
                }
            ),
            "enable_output" => return Some(
                Formula {
                    inp_cell: Cell{row: 0, col: 0},
                    expression: Expression::Enable,
                }
            ),
            "disable_output" => return Some(
                Formula {
                    inp_cell: Cell{row: 0, col: 0},
                    expression: Expression::Disable,
                }
            ),
            _ => panic!("Invalid Sheet Command!"),
        }
    }
    if let Some(caps) = scroll_re.captures(&line) {
        let scroll = &caps["scroll"];

        match scroll {
            "w" => return Some(
                Formula {
                    inp_cell: Cell{row: 0, col: 0},
                    expression: Expression::ScrollUp,
                }),
            "a" => return Some(
                Formula {
                    inp_cell: Cell{row: 0, col: 0},
                    expression: Expression::ScrollLeft,
                }),
            "s" => return Some(
                Formula {
                    inp_cell: Cell{row: 0, col: 0},
                    expression: Expression::ScrollDown,
                }),
            "d" => return Some(
                Formula {
                    inp_cell: Cell{row: 0, col: 0},
                    expression: Expression::ScrollRight,
                }),
            _ => panic!("Invalid Scroll Command!"),
        }
    }
    None
}


#[cfg(test)]
mod formula_tests {
    use super::*;


    fn test_binary_op(inp_cell: &str, op: &str, val1: &str, val2: &str, form: Formula) {
        let input = format!("{inp_cell}={val1}{op}{val2}");

        let mut buf_inp = Cursor::new(input);
        let form_out = get_formula(&mut buf_inp).expect("Incorrect Input");

        assert_eq!(form_out, form);
    }

    #[test]
    fn binary_op () {
        test_binary_op("A1", "+", "-4", "3", 
        Formula { inp_cell: Cell { row: 1, col: 1 }, expression: Expression::Add(Value::Num(-4), Value::Num(3))});

        test_binary_op("ZZZ898", "*", "   -4", "        3\n", 
        Formula { inp_cell: Cell { row: 898, col: 18278 }, expression: Expression::Mul(Value::Num(-4), Value::Num(3))});

        test_binary_op("ZZZ898", "/", "   -4    ", "        3\n", 
        Formula { inp_cell: Cell { row: 898, col: 18278 }, expression: Expression::Div(Value::Num(-4), Value::Num(3))});

        test_binary_op("ZZZ898", "-", "   4    ", "        3\n", 
        Formula { inp_cell: Cell { row: 898, col: 18278 }, expression: Expression::Sub(Value::Num(4), Value::Num(3))});

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

    fn test_num_of_col_of_num(num : u32) {
        let col = Col::from_num(num).expect("Column Number out of Range\n");
        let num_out = col.num_of_col();

        assert_eq!(num, num_out);
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
        test_num_of_col("A",    1);
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
        test_col_of_num("AB",  28);
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