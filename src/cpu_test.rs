use super::*;

#[test]
fn test_boot_state() {
    let cpu = Cpu::new();
    assert_eq!(cpu.pc, 0x200);
    assert_eq!(cpu.sp, 0);
    assert!(!cpu.should_draw);
    assert!(!cpu.should_beep);
}

#[test]
fn test_cls() {
    let mut cpu = Cpu::new();
    cpu.gfx = vec![0xF; 64 * 32];
    cpu.store_16(0x200, 0x00e0);
    cpu.step();

    assert!(cpu.gfx.iter().all(|&x| x == 0));
    assert_eq!(cpu.gfx.len(), 64 * 32);
}

#[test]
fn test_ret() {
    let mut cpu = Cpu::new();
    cpu.sp = 2;
    cpu.stack[1] = 0x1234;
    cpu.store_16(0x200, 0x00ee);
    cpu.step();
    assert_eq!(cpu.sp, 1);
    assert_eq!(cpu.pc, 0x1234);
}

#[test]
fn test_jmp() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x1234);
    cpu.step();
    assert_eq!(cpu.pc, 0x0234);
}

#[test]
fn test_subroutine() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x2345);
    cpu.step();
    assert_eq!(cpu.sp, 1);
    assert_eq!(cpu.stack[0], 0x202);
    assert_eq!(cpu.pc, 0x0345);
}

#[test]
fn test_je() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x3042);
    cpu.step();
    assert_eq!(cpu.pc, 0x202);

    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x3000);
    cpu.step();
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn test_jne() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x4042);
    cpu.step();
    assert_eq!(cpu.pc, 0x204);

    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x4000);
    cpu.step();
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn test_je_regs() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x5120);
    cpu.step();
    assert_eq!(cpu.pc, 0x204);

    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x5120);
    cpu.regs[1] = 0x42;
    cpu.step();
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn test_load() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x6842);
    cpu.step();
    assert_eq!(cpu.regs[8], 0x42);
}

#[test]
fn test_add_value() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x7842);
    cpu.regs[8] = 1;
    cpu.step();
    assert_eq!(cpu.regs[8], 0x43);
}

#[test]
fn test_assign() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x8120);
    cpu.regs[2] = 0x42;
    cpu.step();
    assert_eq!(cpu.regs[1], 0x42);
}

#[test]
fn test_bit_or() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x8121);
    cpu.regs[1] = 0xF0;
    cpu.regs[2] = 0x0F;
    cpu.step();
    assert_eq!(cpu.regs[1], 0xFF);
}

#[test]
fn test_bit_and() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x8122);
    cpu.regs[1] = 0b1101;
    cpu.regs[2] = 0b1011;
    cpu.step();
    assert_eq!(cpu.regs[1], 0b1001);
}

#[test]
fn test_bit_xor() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x8123);
    cpu.regs[1] = 0b1101;
    cpu.regs[2] = 0b1011;
    cpu.step();
    assert_eq!(cpu.regs[1], 0b0110);
}

#[test]
fn test_add_regs() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x8124);
    cpu.regs[1] = 0x10;
    cpu.regs[2] = 0x20;
    cpu.step();
    assert_eq!(cpu.regs[1], 0x30);
    assert_eq!(cpu.regs[0xF], 0); // no carry

    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x8124);
    cpu.regs[1] = 0xFF;
    cpu.regs[2] = 0x2;
    cpu.step();
    assert_eq!(cpu.regs[1], 1);
    assert_eq!(cpu.regs[0xF], 1); // carry
}

#[test]
fn test_sub_xy() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x8125);
    cpu.regs[1] = 20;
    cpu.regs[2] = 15;
    cpu.step();
    assert_eq!(cpu.regs[1], 5);
    assert_eq!(cpu.regs[0xF], 1); // no borrow

    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x8125);
    cpu.regs[1] = 0x0;
    cpu.regs[2] = 0x1;
    cpu.step();
    assert_eq!(cpu.regs[1], 0xFF);
    assert_eq!(cpu.regs[0xF], 0); // borrow
}

#[test]
fn test_shr() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x8126);
    cpu.regs[1] = 0b0110_0101;
    cpu.step();
    assert_eq!(cpu.regs[1], 0b0011_0010);
    assert_eq!(cpu.regs[0xF], 1); // LSB is stored in VF
}

#[test]
fn test_sub_yx() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x8127);
    cpu.regs[1] = 15;
    cpu.regs[2] = 20;
    cpu.step();
    assert_eq!(cpu.regs[1], 5);
    assert_eq!(cpu.regs[0xF], 1); // no borrow

    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x8127);
    cpu.regs[1] = 1;
    cpu.regs[2] = 0;
    cpu.step();
    assert_eq!(cpu.regs[1], 0xFF);
    assert_eq!(cpu.regs[0xF], 0); // borrow
}

#[test]
fn test_shl() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x812E);
    cpu.regs[1] = 0b1000_1101;
    cpu.step();
    assert_eq!(cpu.regs[1], 0b0001_1010);
    assert_eq!(cpu.regs[0xF], 1); // MSB is stored in VF
}

#[test]
fn test_jne_regs() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x9120);
    cpu.step();
    assert_eq!(cpu.pc, 0x202);

    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0x9120);
    cpu.regs[1] = 0x42;
    cpu.step();
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn test_set_i() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0xA123);
    cpu.step();
    assert_eq!(cpu.i, 0x0123);
}

#[test]
fn test_jmp_plus_v0() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0xB123);
    cpu.regs[0] = 1;
    cpu.step();
    assert_eq!(cpu.pc, 0x0124);
}

#[test]
fn test_rand() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0xC000);
    cpu.regs[0] = 0xFF;
    cpu.step();
    // Meh. Random numbers and ANDed with nn so... close enough
    assert_eq!(cpu.regs[0], 0);
}

#[test]
fn test_get_timer() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0xF207);
    cpu.delay = 0x42;
    cpu.step();
    assert_eq!(cpu.regs[2], 0x41);
}

#[test]
fn test_set_timer() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0xF215);
    cpu.regs[2] = 0x42;
    cpu.step();
    assert_eq!(cpu.delay, 0x42);
}

#[test]
fn test_set_sound() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0xF218);
    cpu.regs[2] = 0x42;
    cpu.step();
    assert_eq!(cpu.sound, 0x42);
}

#[test]
fn test_add_reg_to_i() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0xF21E);
    cpu.i = 2;
    cpu.regs[2] = 0x42;
    cpu.step();
    assert_eq!(cpu.i, 0x44);
}

#[test]
fn test_set_i_to_char() {
    let mut cpu = Cpu::new();
    cpu.store_16(0x200, 0xF229);
    cpu.regs[2] = 0xA;
    cpu.step();
    assert_eq!(cpu.i, 0xA * 5); 
}

// TODO:
// DXYN (draw)
// EX9E	if(key()==Vx)
// EXA1	if(key()!=Vx)
// FX0A	Vx = get_key()
// FX33	BCD	set_BCD(Vx);
// FX55	MEM	reg_dump(Vx,&I)
// FX65	MEM	reg_load(Vx,&I)
