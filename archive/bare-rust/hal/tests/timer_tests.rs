#![no_std]

#[cfg(feature = "std")]
extern crate std;

use hal::timer::MicroSeconds;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_no_wrap_around() {
        let time1 = MicroSeconds::new(20000);
        let time2 = MicroSeconds::new(10000);
        let result = time1.sub(time2);
        assert_eq!(result, MicroSeconds::new(10000));
    }

    #[test]
    fn test_sub_with_wrap_around() {
        let max: u64 = 3600 * 1000_000;
        let time1 = MicroSeconds::new(5000);
        let time2 = MicroSeconds::new(max - 5000);
        let result = time1.sub(time2);
        assert_eq!(result, MicroSeconds::new(10001));
    }
}
