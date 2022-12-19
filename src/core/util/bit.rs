#[inline(always)]
pub fn test_bit(val: u8, index: u8) -> bool {
    assert!(index < 8);
    (val >> index) & 0x1 != 0x0
}

#[inline(always)]
pub fn set_bit(val: u8, index: u8) -> u8 {
    assert!(index < 8);
    (0x1 << index) | val
}

#[inline(always)]
pub fn clear_bit(val: u8, index: u8) -> u8 {
    assert!(index < 8);
    !(0x1 << index) & val
}

#[inline(always)]
pub fn extract_bits(val: u8, msb: u8, lsb: u8) -> u8 {
    assert!(msb < 8 && lsb < 8 && msb >= lsb);
    let mut ret = 0;
    let mut mask = 0b1;
    for i in lsb ..= msb {
        if test_bit(val, i) {
            ret |= mask;
        }
        mask <<= 1;
    }
    ret
}

#[cfg(test)]
mod bit_tests {
    use super::*;
    #[test]
    fn bit_ops() {
        let a = 0b0000_1010u8;
        let b = 0b1101_0011u8;

        assert!(!test_bit(a, 0));
        assert!(test_bit(a, 1));
        assert!(test_bit(a, 3));
        assert!(!test_bit(a, 5));
        assert!(!test_bit(a, 7));

        assert!(test_bit(b, 0));
        assert!(test_bit(b, 1));
        assert!(!test_bit(b, 3));
        assert!(!test_bit(b, 5));
        assert!(test_bit(b, 7));

        assert_eq!(set_bit(a, 0), 0b0000_1011u8);
        assert_eq!(set_bit(a, 2), 0b0000_1110u8);
        assert_eq!(set_bit(a, 3), 0b0000_1010u8);
        assert_eq!(set_bit(a, 6), 0b0100_1010u8);
        assert_eq!(set_bit(a, 7), 0b1000_1010u8);
        assert_eq!(set_bit(b, 0), 0b1101_0011u8);
        assert_eq!(set_bit(b, 2), 0b1101_0111u8);
        assert_eq!(set_bit(b, 3), 0b1101_1011u8);
        assert_eq!(set_bit(b, 5), 0b1111_0011u8);
        assert_eq!(set_bit(b, 7), 0b1101_0011u8);

        assert_eq!(clear_bit(a, 0), 0b0000_1010u8);
        assert_eq!(clear_bit(a, 1), 0b0000_1000u8);
        assert_eq!(clear_bit(a, 3), 0b0000_0010u8);
        assert_eq!(clear_bit(a, 6), 0b0000_1010u8);
        assert_eq!(clear_bit(a, 7), 0b0000_1010u8);
        assert_eq!(clear_bit(b, 0), 0b1101_0010u8);
        assert_eq!(clear_bit(b, 1), 0b1101_0001u8);
        assert_eq!(clear_bit(b, 3), 0b1101_0011u8);
        assert_eq!(clear_bit(b, 4), 0b1100_0011u8);
        assert_eq!(clear_bit(b, 7), 0b0101_0011u8);

        assert_eq!(extract_bits(a, 3, 0), 0b1010u8);
        assert_eq!(extract_bits(a, 5, 2), 0b0010u8);
        assert_eq!(extract_bits(a, 7, 6), 0b00u8);
        assert_eq!(extract_bits(a, 1, 1), 0b1u8);
        assert_eq!(extract_bits(a, 7, 7), 0b0u8);
        assert_eq!(extract_bits(b, 3, 0), 0b0011u8);
        assert_eq!(extract_bits(b, 5, 2), 0b0100u8);
        assert_eq!(extract_bits(b, 7, 6), 0b11u8);
        assert_eq!(extract_bits(b, 1, 1), 0b1u8);
        assert_eq!(extract_bits(b, 7, 7), 0b1u8);
    }
}