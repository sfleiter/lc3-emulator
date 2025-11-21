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
