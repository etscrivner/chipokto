extern crate okto;
use okto::cpu::{Cpu, Operation, Instruction, InstructionParts};

#[test]
fn cpu_data_structure() {
    let _cpu = Cpu::new();
}

#[test]
fn instruction_parts() {
    assert_eq!(0x3, (0x8349 as Instruction).vx());
    assert_eq!(0x4, (0x8349 as Instruction).vy());
    assert_eq!(0x349, (0x8349 as Instruction).addr());
    assert_eq!(0x89, (0x3389 as Instruction).imm());
    assert_eq!(0x9, (0x3389 as Instruction).nib());
}

#[test]
fn chip8_opcodes() {
    assert_eq!(Some(Operation::Cls), Operation::from_instruction(&0x00E0));
    assert_eq!(Some(Operation::Ret), Operation::from_instruction(&0x00EE));
    assert_eq!(
        Some(Operation::Sys(0x3FF)),
        Operation::from_instruction(&0x03FF)
    );
    assert_eq!(
        Some(Operation::Jump(0x234)),
        Operation::from_instruction(&0x1234)
    );
    assert_eq!(
        Some(Operation::Call(0xABC)),
        Operation::from_instruction(&0x2ABC)
    );
    assert_eq!(
        Some(Operation::SkipEqImm(0x3, 0x25)),
        Operation::from_instruction(&0x3325)
    );
    assert_eq!(
        Some(Operation::SkipNeqImm(0xF, 0x12)),
        Operation::from_instruction(&0x4F12)
    );
    assert_eq!(
        Some(Operation::SkipEqReg(0x1, 0xA)),
        Operation::from_instruction(&0x51A0)
    );
    assert_eq!(
        Some(Operation::LoadImm(0x4, 0x12)),
        Operation::from_instruction(&0x6412)
    );
    assert_eq!(
        Some(Operation::AddImm(0xB, 0x23)),
        Operation::from_instruction(&0x7B23)
    );

    assert_eq!(
        Some(Operation::LoadReg(0x2, 0x3)),
        Operation::from_instruction(&0x8230)
    );
    assert_eq!(
        Some(Operation::Or(0x3, 0x4)),
        Operation::from_instruction(&0x8341)
    );
    assert_eq!(
        Some(Operation::And(0x5, 0x6)),
        Operation::from_instruction(&0x8562)
    );
    assert_eq!(
        Some(Operation::Xor(0x6, 0x7)),
        Operation::from_instruction(&0x8673)
    );
    assert_eq!(
        Some(Operation::AddReg(0x7, 0x8)),
        Operation::from_instruction(&0x8784)
    );
    assert_eq!(
        Some(Operation::Sub(0x8, 0x9)),
        Operation::from_instruction(&0x8895)
    );
    assert_eq!(
        Some(Operation::Shr(0x8)),
        Operation::from_instruction(&0x8896)
    );
    assert_eq!(
        Some(Operation::SubNeg(0x8, 0x9)),
        Operation::from_instruction(&0x8897)
    );
    assert_eq!(
        Some(Operation::Shl(0x8)),
        Operation::from_instruction(&0x889E)
    );

    assert_eq!(
        Some(Operation::SkipNeqReg(0xB, 0xC)),
        Operation::from_instruction(&0x9BC0)
    );
    assert_eq!(
        Some(Operation::LoadAddr(0xDED)),
        Operation::from_instruction(&0xADED)
    );
    assert_eq!(
        Some(Operation::JumpAddrPlusV0(0xBEF)),
        Operation::from_instruction(&0xBBEF)
    );
    assert_eq!(
        Some(Operation::RandModImm(0xB, 0x3A)),
        Operation::from_instruction(&0xCB3A)
    );
    assert_eq!(
        Some(Operation::Draw(0x1, 0xF, 0x3)),
        Operation::from_instruction(&0xD1F3)
    );

    assert_eq!(
        Some(Operation::SkipKey(0x3)),
        Operation::from_instruction(&0xE39E)
    );
    assert_eq!(
        Some(Operation::SkipNotKey(0x4)),
        Operation::from_instruction(&0xE4A1)
    );

    assert_eq!(
        Some(Operation::LoadRegDelay(0xA)),
        Operation::from_instruction(&0xFA07)
    );
    assert_eq!(
        Some(Operation::WaitKey(0xA)),
        Operation::from_instruction(&0xFA0A)
    );
    assert_eq!(
        Some(Operation::LoadDelayReg(0xC)),
        Operation::from_instruction(&0xFC15)
    );
    assert_eq!(
        Some(Operation::LoadSoundReg(0xF)),
        Operation::from_instruction(&0xFF18)
    );
    assert_eq!(
        Some(Operation::AddAddrReg(0xE)),
        Operation::from_instruction(&0xFE1E)
    );
    assert_eq!(
        Some(Operation::LoadAddrDigit(0x1)),
        Operation::from_instruction(&0xF129)
    );
    assert_eq!(
        Some(Operation::MemStoreBcd(0x4)),
        Operation::from_instruction(&0xF433)
    );
    assert_eq!(
        Some(Operation::MemStoreRegs(0x7)),
        Operation::from_instruction(&0xF755)
    );
    assert_eq!(
        Some(Operation::MemLoadRegs(0x9)),
        Operation::from_instruction(&0xF965)
    );
    assert_eq!(
        Some(Operation::RplStoreRegs(0x7)),
        Operation::from_instruction(&0xF775)
    );
    assert_eq!(
        Some(Operation::RplLoadRegs(0x9)),
        Operation::from_instruction(&0xF985)
    );
}

#[test]
fn superchip8_opcodes() {
    assert_eq!(Some(Operation::Scr), Operation::from_instruction(&0x00FB));
    assert_eq!(Some(Operation::Scl), Operation::from_instruction(&0x00FC));
    assert_eq!(Some(Operation::Exit), Operation::from_instruction(&0x00FD));
    assert_eq!(Some(Operation::Low), Operation::from_instruction(&0x00FE));
    assert_eq!(Some(Operation::High), Operation::from_instruction(&0x00FF));

    assert_eq!(
        Some(Operation::LoadAddrBigDigit(0x5)),
        Operation::from_instruction(&0xF530)
    );
}

#[test]
fn bad_opcodes() {
    // Invalid SkipEqReg
    assert_eq!(None, Operation::from_instruction(&0x51A5));
    // Invalid arithmetic
    assert_eq!(None, Operation::from_instruction(&0x8898));
    assert_eq!(None, Operation::from_instruction(&0x889B));
    assert_eq!(None, Operation::from_instruction(&0x889F));
    // Invalid skip opcodes
    assert_eq!(None, Operation::from_instruction(&0x9AB1));
    assert_eq!(None, Operation::from_instruction(&0x9ABF));
    // Invalid skip key opcodes
    assert_eq!(None, Operation::from_instruction(&0xEA10));
    assert_eq!(None, Operation::from_instruction(&0xEABF));
    // Invalid 0xF000 opcodes
    assert_eq!(None, Operation::from_instruction(&0xFA01));
    assert_eq!(None, Operation::from_instruction(&0xFA20));
    assert_eq!(None, Operation::from_instruction(&0xFAFF));
}
