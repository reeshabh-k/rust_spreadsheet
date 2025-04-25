use arrayvec::ArrayVec;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::BufRead;

use crate::basic::{Cell, Expression, Formula, Value};

#[derive(Debug, Clone, PartialEq)]

/// Represents a column that holds up to 3 `u8` values.
///
/// This struct is a wrapper around an `ArrayVec<u8, 3>`, which is a fixed-size
/// vector that can hold at most 3 elements of type `u8`. It is used when you need
/// to store a small, fixed-size collection of values.
///
/// The `ArrayVec` ensures that the collection is allocated on the stack, making it
/// efficient in terms of memory usage and performance when the number of elements is small.
pub struct Col(pub ArrayVec<u8, 3>);

/// Parses a row string into a `u32` value.
///
/// This function attempts to parse a string slice into an unsigned integer (`u32`).
/// It returns the parsed value wrapped in `Some(u32)` if the number is within the
/// range 1 to 999 (inclusive), or `None` if the value is out of range or the
/// string cannot be parsed into a `u32`.
///
/// # Parameters
///
/// - `row_str`: A string slice that represents the row number to be parsed.
///
/// # Returns
///
/// - `Option<u32>`: Returns `Some(u32)` if the row number is valid and within the
///   allowed range, or `None` if parsing fails or the value is out of range.
fn parse_row(row_str: &str) -> Option<u32> {
    let row_num = row_str.parse::<u32>().ok()?;
    if (1..=999).contains(&row_num) {
        Some(row_num)
    } else {
        None
    }
}

/// Parses a string representation of an integer into an `Option<i32>`.
///
/// This function attempts to convert the input string into a valid `i32` value. If successful,
/// it returns `Some(i32)`. If the conversion fails (e.g., due to an invalid format), it returns `None`.
///
/// # Arguments
/// * `int_str` - A string slice that represents the integer to parse.
///
/// # Returns
/// * `Option<i32>` - `Some(i32)` if the string is a valid integer, otherwise `None`.
fn parse_int(int_str: &str) -> Option<i32> {
    int_str.parse::<i32>().ok()
}

/// A struct representing a column in a spreadsheet, typically identified by a string (e.g., "A", "AB", etc.).
impl Col {
    /// Converts a column string (e.g., "A", "AB") to its corresponding numerical index (e.g., 1, 28).
    ///
    /// The function processes the string as a base-26 numeral system, where 'A' corresponds to 1, 'B' to 2, etc.
    /// The result is a number representing the column's position.
    ///
    /// # Arguments
    /// * `col_str` - A string slice representing the column identifier.
    ///
    /// # Returns
    /// * `Option<u16>` - `Some(u16)` with the column number if valid, otherwise `None` for invalid strings.
    fn from_str_to_num(col_str: &str) -> Option<u16> {
        if col_str.len() > 3 {
            None
        } else {
            let mut val = 0;
            for bt in col_str.as_bytes() {
                if !(&b'A'..=&b'Z').contains(&bt) {
                    return None;
                }
                val *= 26;
                val += ((bt - b'A') as u16) + 1;
            }
            Some(val)
        }
    }

    /// Converts a column string (e.g., "A", "AB") into a `Col` struct.
    ///
    /// # Arguments
    /// * `col_str` - A string slice representing the column identifier.
    ///
    /// # Returns
    /// * `Option<Col>` - `Some(Col)` if valid, otherwise `None` for invalid strings.
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

    /// Converts a column number (e.g., 1, 28) to its corresponding column string (e.g., "A", "AB").
    ///
    /// The function generates the string by performing base-26 arithmetic.
    ///
    /// # Arguments
    /// * `num` - The column number to convert.
    ///
    /// # Returns
    /// * `Option<Col>` - `Some(Col)` with the corresponding column string if the number is valid, otherwise `None`.
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

    /// Returns the numerical index of the column.
    ///
    /// This function calculates the column number by interpreting the bytes of the `Col` struct as a base-26 number.
    ///
    /// # Returns
    /// * `u32` - The numerical index of the column.
    fn num_of_col(&self) -> u32 {
        let mut val: u32 = 0;
        let Col(vec): &Col = self;
        for &bt in vec.iter() {
            val *= 26;
            val += ((bt - b'A') as u32) + 1;
        }
        val
    }

    /// Converts the column to a string representation.
    ///
    /// # Returns
    /// * `&str` - The string representation of the column (e.g., "A", "AB").
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap()
    }
}

// static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"...").unwrap());
// RE.is_match(haystack)

/// Parses a string containing a cell reference (e.g., "A1") into a `Cell` struct.
///
/// The function extracts the column and row parts from the string using regular expressions,
/// and then constructs the corresponding `Cell` object.
///
/// # Arguments
/// * `cell_str` - A string slice representing the cell reference to parse (e.g., "A1").
///
/// # Returns
/// * `Option<Cell>` - `Some(Cell)` if the cell reference is valid, otherwise `None`.
fn parse_cell(cell_str: &str) -> Option<Cell> {
    static CELL_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?P<col>[A-Z]+)(?P<row>[0-9]+)").unwrap());
    let caps = CELL_RE.captures(cell_str)?;

    Some(Cell {
        col: Col::from_str(&caps["col"])?.num_of_col() as u16,
        row: parse_row(&caps["row"])? as u16,
    })
}

/// Attempts to parse a value (either a number or a cell reference) from the given string.
///
/// The function tries to parse the input as a cell reference first, and if that fails, it tries
/// parsing it as an integer. If both fail, `None` is returned.
///
/// # Parameters
/// - `val_str`: A string slice representing the value to parse (could be a number or a cell reference).
///
/// # Returns
/// - `Some(Value)` if parsing succeeds, otherwise `None`.
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

/// Reads and parses a formula from the given reader.
///
/// This function attempts to read a line from the provided `reader` and parse it into
/// a `Formula` object based on predefined patterns (binary operations, range operations,
/// commands, etc.). It returns the parsed formula or `None` if the line doesn't match
/// any known patterns.
///
/// # Parameters
/// - `reader`: A mutable reference to a reader (e.g., a file or stdin) from which to read the formula.
///
/// # Returns
/// - `Some(Formula)` if a valid formula is parsed, `None` if no valid formula is found.
pub fn get_formula<R: BufRead>(reader: &mut R) -> Option<Formula> {
    let mut line = String::new();
    let _bytes_read = reader.read_line(&mut line);

    static BINARY_OP_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(?P<cell>[A-Z]+[0-9]+)\s*=\s*(?P<val1>-?\d+|[A-Z]+[0-9]+)\s*(?P<op>[*/+-])\s*(?P<val2>-?\d+|[A-Z]+[0-9]+)\s*$").unwrap()
    });
    static RANGE_OP_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(?P<cell>[A-Z]+[0-9]+)\s*=\s*(?P<op>MAX|MIN|STDEV|AVG|SUM)\s*['(']\s*(?P<cell1>[A-Z]+[0-9]+)\s*:\s*(?P<cell2>[A-Z]+[0-9]+)\s*[')']\s*$").unwrap()
    });
    static SLEEP_OP_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(?P<cell>[A-Z]+[0-9]+)\s*=\s*SLEEP\s*['(']\s*(?P<val>-?\d+|[A-Z]+[0-9]+)\s*[')']\s*$").unwrap()
    });
    static CONSTANT_OP_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"^(?P<cell_col>[A-Z]+)(?P<cell_row>[0-9]+)\s*=\s*(?P<val>-?\d+|[A-Z]+[0-9]+)\s*$",
        )
        .unwrap()
    });
    static COMMANDS_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\s*(?P<command>q|enable_output|disable_output)\s*$").unwrap());
    static SCROLL_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\s*(?P<scroll>w|a|s|d)\s*$").unwrap());
    static SCROLL_TO_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\s*scroll_to\s*(?P<cell>[A-Z]+[0-9]+)\s*$").unwrap());

    if let Some(caps) = CONSTANT_OP_RE.captures(&line) {
        let cell_col = Col::from_str_to_num(&caps["cell_col"])?;
        let cell_row: u16 = caps["cell_row"].parse::<u16>().ok()?;
        let val = parse_val(&caps["val"])?;

        let form = Formula {
            inp_cell: Cell {
                col: cell_col,
                row: cell_row,
            },
            expression: Expression::Constant(val),
        };

        return Some(form);
    }

    if let Some(caps) = SCROLL_TO_RE.captures(&line) {
        let cell = parse_cell(&caps["cell"])?;

        let form = Formula {
            inp_cell: Cell { row: 0, col: 0 },
            expression: Expression::ScrollTo(cell),
        };

        return Some(form);
    }

    if let Some(caps) = BINARY_OP_RE.captures(&line) {
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
    if let Some(caps) = RANGE_OP_RE.captures(&line) {
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
    if let Some(caps) = SLEEP_OP_RE.captures(&line) {
        let cell = parse_cell(&caps["cell"])?;
        let val = parse_val(&caps["val"])?;

        let form = Formula {
            inp_cell: cell,
            expression: Expression::Sleep(val),
        };

        return Some(form);
    }
    if let Some(caps) = COMMANDS_RE.captures(&line) {
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
    if let Some(caps) = SCROLL_RE.captures(&line) {
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
                expression: Expression::ScrollTo(Cell {
                    row: 898,
                    col: 18278,
                }),
            },
        );
    }

    #[test]
    fn test_get_formula() {
        let input = "A1=B2";
        let mut buf_inp = Cursor::new(input);
        let form_out = get_formula(&mut buf_inp).expect("Incorrect Input");
        let form_exp = Formula {
            inp_cell: Cell { row: 1, col: 1 },
            expression: Expression::Constant(Value::Ref(Cell { row: 2, col: 2 })),
        };
        assert_eq!(form_out, form_exp);
    }

    #[test]
    fn test_get_formula_invalid() {
        let input = "A1=B2#C2";
        let mut buf_inp = Cursor::new(input);
        let form_out = get_formula(&mut buf_inp);
        assert_eq!(form_out, None);
    }

    #[test]
    fn test_get_formula_invalid2() {
        let input = "A1=B2+";
        let mut buf_inp = Cursor::new(input);
        let form_out = get_formula(&mut buf_inp);
        assert_eq!(form_out, None);
    }
    #[test]
    fn test_range_invalid() {
        let input = "A1=MAXin(A2:B3)";
        let mut buf_inp = Cursor::new(input);
        let form_out = get_formula(&mut buf_inp);
        assert_eq!(form_out, None);
    }

    #[test]
    // #[should_panic]
    fn test_invalid_scroll() {
        let input = "ww";
        let mut buf_inp = Cursor::new(input);
        let form_out = get_formula(&mut buf_inp);
        assert_eq!(form_out, None);
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
                expression: Expression::Max(Cell { row: 2, col: 1 }, Cell { row: 3, col: 2 }),
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
                expression: Expression::Min(Cell { row: 2, col: 1 }, Cell { row: 3, col: 2 }),
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
                expression: Expression::Avg(Cell { row: 2, col: 1 }, Cell { row: 3, col: 2 }),
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
                expression: Expression::Stdev(Cell { row: 2, col: 1 }, Cell { row: 3, col: 2 }),
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
                expression: Expression::Sum(Cell { row: 2, col: 1 }, Cell { row: 3, col: 2 }),
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
                inp_cell: Cell { row: 0, col: 0 },
                expression: Expression::Enable,
            },
        );

        test_command_op(
            "disable_output",
            Formula {
                inp_cell: Cell { row: 0, col: 0 },
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
                inp_cell: Cell { row: 0, col: 0 },
                expression: Expression::ScrollLeft,
            },
        );

        test_scroll_op(
            "s",
            Formula {
                inp_cell: Cell { row: 0, col: 0 },
                expression: Expression::ScrollDown,
            },
        );

        test_scroll_op(
            "d",
            Formula {
                inp_cell: Cell { row: 0, col: 0 },
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
                inp_cell: Cell { row: 0, col: 0 },
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

    fn test_str_to_num(col_str: &str, num: u16) {
        let col_out = Col::from_str_to_num(col_str).expect("Invalid Column String");
        assert_eq!(col_out, num);
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
    fn num_of_str() {
        test_str_to_num("A", 1);
        test_str_to_num("D", 4);
        test_str_to_num("AB", 28);
        test_str_to_num("MA", 339);
        test_str_to_num("YE", 655);
        test_str_to_num("DEF", 2840);
        test_str_to_num("ZKM", 17875);
        test_str_to_num("ZZZ", 18278);
    }

    #[test]
    #[should_panic]
    fn num_of_str_out_of_range() {
        test_str_to_num("AAAA", 0);
    }

    #[test]
    #[should_panic]
    fn num_of_str_invalid() {
        test_str_to_num("A+", 0);
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

    #[test]
    fn test_from_num() {
        for i in 18279..=100000 {
            let col = Col::from_num(i);
            assert_eq!(col, None);
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
    fn test_parse_val_is_none(val_str: &str) {
        let val_out = parse_val(val_str);
        assert_eq!(val_out, None);
    }

    #[test]
    fn parse_cell_test() {
        test_parse_cell("A1", Cell { col: 1, row: 1 });
        test_parse_cell(
            "ZZZ898",
            Cell {
                col: 18278,
                row: 898,
            },
        );
        test_parse_cell("AB10", Cell { col: 28, row: 10 });
        test_parse_cell("MA339", Cell { col: 339, row: 339 });
        test_parse_cell("YE655", Cell { col: 655, row: 655 });
        test_parse_cell(
            "DEF280",
            Cell {
                col: 2840,
                row: 280,
            },
        );
        test_parse_cell(
            "ZKM175",
            Cell {
                col: 17875,
                row: 175,
            },
        );
        test_parse_cell(
            "ZZZ188",
            Cell {
                col: 18278,
                row: 188,
            },
        );
    }

    #[test]
    fn parse_val_test() {
        test_parse_val("A1", Value::Ref(Cell { col: 1, row: 1 }));
        test_parse_val(
            "ZZZ898",
            Value::Ref(Cell {
                col: 18278,
                row: 898,
            }),
        );
        test_parse_val("-4", Value::Num(-4));
        test_parse_val("3", Value::Num(3));
        test_parse_val("0", Value::Num(0));
        test_parse_val("100", Value::Num(100));
    }

    #[test]
    fn test_parse_val2() {
        test_parse_val_is_none("A+A");
    }
}
