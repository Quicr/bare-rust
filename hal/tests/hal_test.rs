#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(test)]
mod tests {

    #[test]
    fn test_init() {
        let freq = 16_000_000;

        hal::init(freq);

        hal::validate();
        //assert!(false);
    }
}
