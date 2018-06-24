extern crate okto;

use okto::cpu;
use okto::keyboard;
use okto::machine::Machine;
use okto::{OktoError, OktoErrorKind};

#[test]
fn machine_initialization() {
    let machine = Machine::new(Box::new(keyboard::nop_wait_key_callback));

    // Initial CPU state
    assert!(machine.cpu.v.iter().all(|&x| x == 0));
    assert_eq!(machine.cpu.i, 0);
    assert_eq!(machine.cpu.pc, cpu::DEFAULT_PC_ADDRESS);
    assert_eq!(machine.cpu.sp, 0);
    assert!(machine.cpu.stack.iter().all(|&x| x == 0));

    // Initial memory state
    assert!(machine.memory.data[0x200..].iter().all(|&x| x == 0));

    // Initial keyboard state
    assert!(
        machine
            .keyboard
            .keys
            .iter()
            .all(|&x| x == keyboard::KeyState::Released)
    );

    // Initial display state
    assert!(
        machine
            .display
            .data
            .iter()
            .all(|&y| y.iter().all(|&x| x == 0))
    );
    assert!(!machine.display.high_resolution);

    // Initial sound state
    assert_eq!(machine.sound.timer, 0);
}

#[test]
fn chip8_cpu_operations() {
    let mut machine = Machine::new(Box::new(keyboard::nop_wait_key_callback));

    // Ret
    assert_eq!(
        Some(OktoError::new(OktoErrorKind::StackUnderflow)),
        machine.execute(cpu::Operation::Ret).err()
    );

    machine.cpu.push_stack(0x2F3).unwrap();
    machine.execute(cpu::Operation::Ret).unwrap();
    assert_eq!(0x2F3, machine.cpu.pc);

    // Jump
    machine.execute(cpu::Operation::Jump(0xABC)).unwrap();
    assert_eq!(0xABC, machine.cpu.pc);

    machine.cpu.v[0] = 0x3;
    machine
        .execute(cpu::Operation::JumpAddrPlusV0(0x233))
        .unwrap();
    assert_eq!(0x236, machine.cpu.pc);

    // Call
    machine.cpu.pc = 0x200;
    machine.execute(cpu::Operation::Call(0x233)).unwrap();
    assert_eq!(0x233, machine.cpu.pc);
    assert_eq!(1, machine.cpu.sp);
    assert_eq!(Some(0x200), machine.cpu.pop_stack());

    // SkipEqImm
    machine.cpu.pc = 0x200;
    machine.cpu.v[0x1] = 0x23;
    machine
        .execute(cpu::Operation::SkipEqImm(0x1, 0x23))
        .unwrap();
    assert_eq!(0x202, machine.cpu.pc);

    machine.cpu.pc = 0x200;
    machine.cpu.v[0x1] = 0x23;
    machine
        .execute(cpu::Operation::SkipEqImm(0x1, 0x01))
        .unwrap();
    assert_eq!(0x200, machine.cpu.pc);

    // SkipEqReg
    machine.cpu.pc = 0x200;
    machine.cpu.v[0x1] = 0x11;
    machine.cpu.v[0x5] = 0x11;
    machine
        .execute(cpu::Operation::SkipEqReg(0x1, 0x5))
        .unwrap();
    assert_eq!(0x202, machine.cpu.pc);

    machine.cpu.pc = 0x200;
    machine.cpu.v[0x1] = 0x11;
    machine.cpu.v[0x5] = 0x12;
    machine
        .execute(cpu::Operation::SkipEqReg(0x1, 0x5))
        .unwrap();
    assert_eq!(0x200, machine.cpu.pc);

    // SkipNeqImm
    machine.cpu.pc = 0x200;
    machine.cpu.v[0x3] = 0x33;
    machine
        .execute(cpu::Operation::SkipNeqImm(0x3, 0x13))
        .unwrap();
    assert_eq!(0x202, machine.cpu.pc);

    machine.cpu.pc = 0x200;
    machine.cpu.v[0x3] = 0x33;
    machine
        .execute(cpu::Operation::SkipNeqImm(0x3, 0x33))
        .unwrap();
    assert_eq!(0x200, machine.cpu.pc);

    // SkipNeqReg
    machine.cpu.pc = 0x200;
    machine.cpu.v[0x1] = 0x11;
    machine.cpu.v[0x2] = 0x23;
    machine
        .execute(cpu::Operation::SkipNeqReg(0x1, 0x2))
        .unwrap();
    assert_eq!(0x202, machine.cpu.pc);

    machine.cpu.pc = 0x200;
    machine.cpu.v[0x1] = 0x11;
    machine.cpu.v[0x2] = 0x11;
    machine
        .execute(cpu::Operation::SkipNeqReg(0x1, 0x2))
        .unwrap();
    assert_eq!(0x200, machine.cpu.pc);

    // LoadImm
    machine.cpu.v[0x1] = 0x00;
    machine.execute(cpu::Operation::LoadImm(0x1, 0x45)).unwrap();
    assert_eq!(0x45, machine.cpu.v[0x1]);

    // LoadReg
    machine.cpu.v[0xA] = 0x00;
    machine.cpu.v[0xB] = 0x12;
    machine.execute(cpu::Operation::LoadReg(0xA, 0xB)).unwrap();
    assert_eq!(0x12, machine.cpu.v[0xA]);
    assert_eq!(0x12, machine.cpu.v[0xB]);

    // LoadAddr
    machine.cpu.i = 0x000;
    machine.execute(cpu::Operation::LoadAddr(0x123)).unwrap();
    assert_eq!(0x123, machine.cpu.i);

    // LoadAddrDigit
    machine.cpu.v[0x5] = 0x3;
    machine.execute(cpu::Operation::LoadAddrDigit(0x5)).unwrap();
    assert_eq!(
        machine.memory.sprite_address_for_digit(0x3).unwrap(),
        machine.cpu.i
    );

    machine.cpu.v[0x5] = 0x10;
    assert_eq!(
        Some(OktoError::new(OktoErrorKind::InvalidDigitSprite)),
        machine.execute(cpu::Operation::LoadAddrDigit(0x5)).err()
    );

    // AddImm
    machine.cpu.v[0x2] = 11;
    machine.execute(cpu::Operation::AddImm(0x2, 5)).unwrap();
    assert_eq!(16, machine.cpu.v[0x2]);

    // AddReg
    machine.cpu.v[0x2] = 10;
    machine.cpu.v[0x3] = 15;
    machine.execute(cpu::Operation::AddReg(0x2, 0x3)).unwrap();
    assert_eq!(25, machine.cpu.v[0x2]);
    assert_eq!(0, machine.cpu.v[0xF]);

    // AddReg - Overflow
    machine.cpu.v[0x2] = 0xFE;
    machine.cpu.v[0x3] = 0x03;
    machine.execute(cpu::Operation::AddReg(0x2, 0x3)).unwrap();
    assert_eq!(0x01, machine.cpu.v[0x2]);
    assert_eq!(0x01, machine.cpu.v[0xF]);

    // AddAddrReg
    machine.cpu.v[0x7] = 0xC;
    machine.cpu.i = 0x200;
    machine.execute(cpu::Operation::AddAddrReg(0x7)).unwrap();
    assert_eq!(0x20C, machine.cpu.i);

    // Sub
    machine.cpu.v[0x1] = 5;
    machine.cpu.v[0x2] = 3;
    machine.execute(cpu::Operation::Sub(0x1, 0x2)).unwrap();
    assert_eq!(2, machine.cpu.v[0x1]);
    assert_eq!(1, machine.cpu.v[0xF]);

    // Sub - Underflow
    machine.cpu.v[0x1] = 3;
    machine.cpu.v[0x2] = 5;
    machine.execute(cpu::Operation::Sub(0x1, 0x2)).unwrap();
    assert_eq!(0xFE, machine.cpu.v[0x1]);
    assert_eq!(0, machine.cpu.v[0xF]);

    // SubNeg
    machine.cpu.v[0x1] = 3;
    machine.cpu.v[0x2] = 5;
    machine.execute(cpu::Operation::SubNeg(0x1, 0x2)).unwrap();
    assert_eq!(2, machine.cpu.v[0x1]);
    assert_eq!(1, machine.cpu.v[0xF]);

    // SubNeg - Underflow
    machine.cpu.v[0x1] = 5;
    machine.cpu.v[0x2] = 3;
    machine.execute(cpu::Operation::SubNeg(0x1, 0x2)).unwrap();
    assert_eq!(0xFE, machine.cpu.v[0x1]);
    assert_eq!(0, machine.cpu.v[0xF]);

    // Or
    machine.cpu.v[0x1] = 0x1F;
    machine.cpu.v[0x2] = 0x23;
    machine.execute(cpu::Operation::Or(0x1, 0x2)).unwrap();
    assert_eq!(0x3F, machine.cpu.v[0x1]);

    // And
    machine.cpu.v[0x1] = 0x1F;
    machine.cpu.v[0x2] = 0x33;
    machine.execute(cpu::Operation::And(0x1, 0x2)).unwrap();
    assert_eq!(0x13, machine.cpu.v[0x1]);

    // Xor
    machine.cpu.v[0x1] = 0x1F;
    machine.cpu.v[0x2] = 0x23;
    machine.execute(cpu::Operation::Xor(0x1, 0x2)).unwrap();
    assert_eq!(0x3C, machine.cpu.v[0x1]);

    // Shr - Bit is 0
    machine.cpu.v[0x1] = 0x0C;
    machine.execute(cpu::Operation::Shr(0x1)).unwrap();
    assert_eq!(0x06, machine.cpu.v[0x1]);
    assert_eq!(0, machine.cpu.v[0xF]);

    // Shr - Bit is 1
    machine.cpu.v[0x1] = 0x03;
    machine.execute(cpu::Operation::Shr(0x1)).unwrap();
    assert_eq!(0x01, machine.cpu.v[0x1]);
    assert_eq!(1, machine.cpu.v[0xF]);

    // Shl - Bit is 0
    machine.cpu.v[0x1] = 0x10;
    machine.execute(cpu::Operation::Shl(0x1)).unwrap();
    assert_eq!(0x20, machine.cpu.v[0x1]);
    assert_eq!(0, machine.cpu.v[0xF]);

    // Shl - Bit is 1
    machine.cpu.v[0x1] = 0xC0;
    machine.execute(cpu::Operation::Shl(0x1)).unwrap();
    assert_eq!(0x80, machine.cpu.v[0x1]);
    assert_eq!(1, machine.cpu.v[0xF]);

    // RandModImm
    machine.cpu.v[0x1] = 0x1F;
    machine
        .execute(cpu::Operation::RandModImm(0x1, 0x2))
        .unwrap();
    assert!(machine.cpu.v[0x1] < 2);
}

#[test]
fn chip8_memory_operations() {
    let mut machine = Machine::new(Box::new(keyboard::nop_wait_key_callback));

    // MemStoreBcd
    machine.cpu.i = 0x200;
    machine.cpu.v[0xB] = 253;
    machine.execute(cpu::Operation::MemStoreBcd(0xB)).unwrap();
    assert_eq!(2, machine.memory.data[0x200]);
    assert_eq!(5, machine.memory.data[0x201]);
    assert_eq!(3, machine.memory.data[0x202]);

    // MemStoreRegs
    machine.cpu.i = 0x200;
    machine.cpu.v[0..5].clone_from_slice(&[0x1, 0x2, 0x3, 0x4, 0x5]);
    machine.execute(cpu::Operation::MemStoreRegs(0x5)).unwrap();
    assert_eq!(
        &machine.memory.data[0x200..0x205],
        &[0x1, 0x2, 0x3, 0x4, 0x5]
    );

    // MemLoadRegs
    machine.cpu.i = 0x200;
    machine.memory.data[0x200..0x205].clone_from_slice(&[0x2, 0x3, 0x4, 0x5, 0x6]);
    machine.execute(cpu::Operation::MemLoadRegs(0x5)).unwrap();
    assert_eq!(&machine.cpu.v[0x0..0x5], &[0x2, 0x3, 0x4, 0x5, 0x6]);
}

#[test]
fn chip8_timer_operations() {
    let mut machine = Machine::new(Box::new(keyboard::nop_wait_key_callback));

    // LoadRegDelay
    machine.delay_timer.value = 0x23;
    assert_eq!(0, machine.cpu.v[0x2]);
    machine.execute(cpu::Operation::LoadRegDelay(0x2)).unwrap();
    assert_eq!(0x23, machine.cpu.v[0x2]);

    // LoadDelayReg
    machine.delay_timer.value = 0;
    machine.cpu.v[0x5] = 0x12;
    machine.execute(cpu::Operation::LoadDelayReg(0x5)).unwrap();
    assert_eq!(0x12, machine.delay_timer.value);
}

#[test]
fn chip8_video_operations() {
    let mut machine = Machine::new(Box::new(keyboard::nop_wait_key_callback));

    // Draw - Pixels not overwritten
    machine.memory.data[0x202] = 0xFF;
    machine.memory.data[0x203] = 0x1F;
    machine.cpu.i = 0x202;
    machine.cpu.v[0x0] = 10;
    machine.cpu.v[0xA] = 10;
    machine.execute(cpu::Operation::Draw(0x0, 0xA, 2)).unwrap();
    assert_eq!(0x00, machine.cpu.v[0xF]);
    assert_eq!(&machine.display.data[10][10..18], &[1, 1, 1, 1, 1, 1, 1, 1]);
    assert_eq!(&machine.display.data[11][10..18], &[0, 0, 0, 1, 1, 1, 1, 1]);

    // Draw - Pixels overwritten
    machine.execute(cpu::Operation::Draw(0x0, 0xA, 2)).unwrap();
    assert_eq!(0x01, machine.cpu.v[0xF]);
    assert_eq!(&machine.display.data[10][10..18], &[0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&machine.display.data[11][10..18], &[0, 0, 0, 0, 0, 0, 0, 0]);

    // Draw - Wrap around
    machine.cpu.v[0x0] = (machine.display.width() - 2) as u8;
    machine.cpu.v[0xA] = (machine.display.height() - 1) as u8;
    machine.execute(cpu::Operation::Draw(0x0, 0xA, 2)).unwrap();
    assert_eq!(0x00, machine.cpu.v[0xF]);
    assert_eq!(
        &machine.display.data[machine.display.height() - 1]
            [machine.display.width() - 2..machine.display.width()],
        &[1, 1]
    );
    assert_eq!(
        &machine.display.data[machine.display.height() - 1][0..6],
        &[1, 1, 1, 1, 1, 1]
    );

    assert_eq!(
        &machine.display.data[0][machine.display.width() - 2..machine.display.width()],
        &[0, 0]
    );
    assert_eq!(&machine.display.data[0][0..6], &[0, 1, 1, 1, 1, 1]);
}

#[test]
fn chip8_keyboard_operations() {
    let mut machine = Machine::new(Box::new(keyboard::nop_wait_key_callback));

    // SkipKey - Key pressed
    machine.keyboard.keys[3] = keyboard::KeyState::Pressed;
    machine.cpu.v[0xA] = 3;
    machine.cpu.pc = 0x200;
    machine.execute(cpu::Operation::SkipKey(0xA)).unwrap();
    assert_eq!(0x202, machine.cpu.pc);

    // SkipKey - Key not pressed
    machine.keyboard.keys[3] = keyboard::KeyState::Released;
    machine.cpu.v[0xA] = 3;
    machine.cpu.pc = 0x200;
    machine.execute(cpu::Operation::SkipKey(0xA)).unwrap();
    assert_eq!(0x200, machine.cpu.pc);

    // SkipNotKey - Key pressed
    machine.keyboard.keys[3] = keyboard::KeyState::Pressed;
    machine.cpu.v[0xA] = 3;
    machine.cpu.pc = 0x200;
    machine.execute(cpu::Operation::SkipNotKey(0xA)).unwrap();
    assert_eq!(0x200, machine.cpu.pc);

    // SkipNotKey - Key not pressed
    machine.keyboard.keys[3] = keyboard::KeyState::Released;
    machine.cpu.v[0xA] = 3;
    machine.cpu.pc = 0x200;
    machine.execute(cpu::Operation::SkipNotKey(0xA)).unwrap();
    assert_eq!(0x202, machine.cpu.pc);

    // WaitKey
    let wait_key_callback = || -> keyboard::WaitKeyResult<u8> { Ok(12) };
    let mut machine = Machine::new(Box::new(wait_key_callback));

    machine.cpu.v[0xA] = 0;
    machine.execute(cpu::Operation::WaitKey(0xA)).unwrap();
    assert_eq!(12, machine.cpu.v[0xA]);
}

#[test]
fn chip8_sound_operations() {
    let mut machine = Machine::new(Box::new(keyboard::nop_wait_key_callback));

    // LoadSoundReg
    machine.cpu.v[0x2] = 0xAB;
    machine.execute(cpu::Operation::LoadSoundReg(0x2)).unwrap();
    assert_eq!(0xAB, machine.sound.timer);
}
