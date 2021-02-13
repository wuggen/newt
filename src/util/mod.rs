pub mod env;
pub mod sh;

/// Get the number of decimal digits in the given number.
pub fn digits(mut num: usize) -> usize {
    let mut res = 0;
    loop {
        res += 1;
        num /= 10;
        if num == 0 {
            break res;
        }
    }
}
