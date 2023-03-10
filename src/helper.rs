pub mod math {
    pub fn log2_u64_with_decimal(x: u64) -> u64 {
        let mut integer = 0;
        let mut t = x;
        let mut m = 1;
        while t > 1 {
            t >>= 1;
            m <<= 1;
            integer += 1;
        }
        let mut fractional = 0;
        m *= 1000;
        t = x * 1000;
        let mut step = m * 71773463 / 1000000000;
        while fractional < 10 {
            m += step;
            if m > t {
                break;
            } else {
                step = m * 71773463 / 1000000000;
                fractional = fractional + 1;
            };
        }
        integer * 10 + fractional
    }

    pub fn sqrt(y: u128)-> u64 {
        if y < 4 {
            if y == 0 {
                0u64
            } else {
                1u64
            }
        } else {
            let mut z = y;
            let mut x = y / 2 + 1;
            while x < z {
                z = x;
                x = (y / x + x) / 2;
            };
            z as u64
        }
    }

    #[test]
    fn test_log2_u64_with_decimal() {
        assert_eq!(log2_u64_with_decimal(0),0);
        assert_eq!(log2_u64_with_decimal(1),0);
        assert_eq!(log2_u64_with_decimal(2),10);
        assert_eq!(log2_u64_with_decimal(3),15);
        assert_eq!(log2_u64_with_decimal(25),46);
        assert_eq!(log2_u64_with_decimal(1024),100);
        assert_eq!(log2_u64_with_decimal(123143400),268);
    }
}
