pub fn twos_complement_to_decimal(bin_rep: u16) -> i16 {
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
    if decimal >= 0 {
        decimal
            .try_into()
            .expect("decimal too large to fit into i16 when computing two's complement")
    } else {
        (!decimal.abs() + 1).cast_unsigned()
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
