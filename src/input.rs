use arrayvec::ArrayVec;
use std::io::{self, BufRead, Error};


#[derive(Debug)]
struct Cell {
    row: u32,
    col: u32,
}

#[derive(Debug)]
enum Value {
    Num(i32),
    Ref(Cell),
}

#[derive(Debug)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Col(ArrayVec<u8, 3>);

impl Col {
    fn from_str(col_str: &str) -> Col {
        if col_str.len() > 3 {
            panic!("Not a valid column string!");
        } else {
            for bt in col_str.as_bytes() {
                if bt < &b'A' || bt > &b'Z' {
                    panic!("Not an accepted character!");
                } 
            }
            let vec: ArrayVec<u8, 3> = ArrayVec::from_iter(col_str.bytes());
            Col(vec)
        }
    }

    fn from_num (mut num : u32) -> Col {
        if (num <= 0 || num > 18278) {
            panic!("Not a valid column number!");
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
            Col(col_out)
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
}


pub fn get_formula<R: BufRead>(reader: & mut R) -> Result<Formula, Error> {
    let line = String::new();
    let _bytes_read = reader.read_line(&mut line)?;
    
    
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
        let col = Col::from_str(col_str);
        let num_exp = col.num_of_col();

        assert_eq!(num_exp, num);
    }

    fn test_num_of_col_of_num(num : u32) {
        let col = Col::from_num(num);
        let num_out = col.num_of_col();

        assert_eq!(num, num_out);
    }

    #[test]
    fn create_a() {
        let col_str = "A";
        let col_out = Col::from_str(col_str);
        let mut col_exp: ArrayVec<u8, 3> = ArrayVec::<u8, 3>::new();
        col_exp.push(b'A');
        let col_exp = Col(col_exp);

        assert_eq!(col_out, col_exp);
    }

    #[test]
    fn create_gf() {
        let col_str = "GF";
        let col_out = Col::from_str(col_str);
        let mut col_exp: ArrayVec<u8, 3> = ArrayVec::new();
        col_exp.push(b'G');
        col_exp.push(b'F');

        let col_exp = Col(col_exp);

        assert_eq!(col_out, col_exp);
    }

    #[test]
    fn create_zzz() {
        let col_str = "ZZZ";
        let col_out = Col::from_str(col_str);
        let col_exp: ArrayVec<u8, 3> = ArrayVec::from([b'Z'; 3]);
        let col_exp = Col(col_exp);

        assert_eq!(col_out, col_exp);
    }


    #[test]
    #[should_panic]
    fn create_aaaa() {
        let col_str = "AAAA";
        let _col_out = Col::from_str(col_str);
    }

    #[test]
    #[should_panic]
    fn create_lower_a() {
        let col_str = "a";
        let _col_out = Col::from_str(col_str);
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
