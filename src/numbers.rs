pub fn twos_complement_to_decimal(bin_rep: u16) -> i16 {
    // we could simply return bin_rep as i16
    // but this is a journey to low level programming, so let's implement this ourselves
    let is_negative = bin_rep >> 15 & 1 == 1;
    #[expect(
        clippy::cast_possible_wrap,
        reason = "Nature of 2's complement is that if leftmost bit i 0, than we cannot overflow"
    )]
    if is_negative {
        let negative_msb_value: i32 = -(1 << 15);
        let res_i32 = (i32::from(bin_rep) & (!(1 << 15))) + negative_msb_value;
        i16::try_from(res_i32).expect("overflow in pc_offset")
    } else {
        bin_rep as i16
    }
}

pub fn decimal_to_twos_complement(decimal: i16) -> u16 {
    // we could simply return decimal as u16
    // but this is a journey to low level programming, so let's implement this ourselves
    if decimal >= 0 {
        decimal.try_into().expect(
            "positive i16 value too large to fit into u16 ??? when computing two's complement",
        )
    } else {
        // avoid overflow during abs() for i16::MIN
        let abs_val: i32 = i32::from(decimal).abs();
        #[expect(clippy::cast_sign_loss)]
        #[expect(
            clippy::cast_possible_truncation,
            reason = "Required by algorithm, see https://en.wikipedia.org/wiki/Two%27s_complement#Procedure"
        )]
        {
            (!abs_val + 1) as u16
        }
    }
}

/// Implements sign extension as described at [Sign extension](https://en.wikipedia.org/wiki/Sign_extension).
pub const fn sign_extend(bits: u16, valid_bits: u8) -> u16 {
    let most_significant_bit = bits >> (valid_bits - 1);
    if most_significant_bit == 1 {
        // negative: 1-extend
        bits | (0xFFFF << valid_bits)
    } else {
        // positive, already 0-extended
        bits
    }
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;
    use yare::parameterized;

    use super::*;

    #[parameterized(
     min = { i16::MIN, 0b1000_0000_0000_0000 },
     almost_min = { i16::MIN+1, 0b1000_0000_0000_0001 },
     zero = { 0, 0b0000_0000_0000_0000 },
     one = { 1, 0b0000_0000_0000_0001 },
     almost_max = { i16::MAX-1, 0b0111_1111_1111_1110 },
     max = { i16::MAX, 0b0111_1111_1111_1111 },
    )]
    #[test_macro(gtest)]
    fn test_decimal_to_twos_complement(input: i16, expected: u16) {
        expect_that!(decimal_to_twos_complement(input), eq(expected));
    }

    #[parameterized(
     min = { 0b1000_0000_0000_0000, i16::MIN },
     almost_min = { 0b1000_0000_0000_0001, i16::MIN+1 },
     zero = { 0b0000_0000_0000_0000, 0 },
     one = { 1, 0b0000_0000_0000_0001 },
     almost_max = { 0b0111_1111_1111_1110, i16::MAX-1 },
     max = { 0b0111_1111_1111_1111, i16::MAX },
    )]
    #[test_macro(gtest)]
    fn test_twos_complement_to_decimal(input: u16, expected: i16) {
        expect_that!(twos_complement_to_decimal(input), eq(expected));
    }
}
