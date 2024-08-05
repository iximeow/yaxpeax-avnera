use yaxpeax_arch::Decoder;

fn test_display(data: &[u8], expected: &'static str) {
    let mut reader = yaxpeax_arch::U8Reader::new(data);
    match yaxpeax_avnera::InstDecoder::default().decode(&mut reader) {
        Ok(instr) => {
            let displayed = instr.to_string();
            assert_eq!(&displayed, expected);
            assert_eq!(data.len() as u8, instr.len());
        }
        Err(e) => {
            panic!("failed to decode {:02x?}: {}", data, e);
        }
    }
}

#[test]
fn test_disassembly() {
    test_display(&[0xc9, 0xf2, 0xed], "[0xedf2] <- r1");
    test_display(&[0xc8, 0x0b, 0x11], "[0x110b] <- r0");
    test_display(&[0xe4, 0x0e], "r4 <- 0x0e");
    test_display(&[0xbc, 0x8a, 0xd9], "jmp 0xd98a");
    test_display(&[0xb9], "ret");
    test_display(&[0x29], "r0 ^= r1");
    test_display(&[0x49], "sbc r0, r1");
    test_display(&[0x59], "scf");
    test_display(&[0x5a], "op5xhi 0x02");
    test_display(&[0x90, 0x50], "jnz $+0x50");
    test_display(&[0x0a], "adc r0, r2");
    test_display(&[0x84], "push r4");
    test_display(&[0xc4], "incw r4:r5");
}
