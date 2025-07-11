//! A simple Fibonacci function.


#[inline(never)]
/// calculate the x'th Fibonacci number
pub fn fib(x: usize) -> u32 {
    if x > 2 {
        fib(x - 1) + fib(x - 2)
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fib_sanity_check() {
        let result = fib(6);
        assert_eq!(result, 8);
    }
}
