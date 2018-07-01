//! Chip8 CPU data structures and types
use super::{OktoError, OktoErrorKind, OktoResult};

/// Chip8 memory address type
pub type Address = u16;
/// Chip8 register type
pub type Register = u8;
/// Chip8 opcode immediate value type
pub type Immediate = u8;
/// Chip8 opcode nibble value type
pub type Nibble = u8;
/// Chip8 instruction type
pub type Instruction = u16;

/// The number of registers in the Chip8 CPU.
pub const NUM_REGISTERS: usize = 16;
/// The maximum number of items allowed on the stack.
pub const MAX_NUM_STACK_ITEMS: usize = 16;
/// The default address for the CPU program counter.
pub const DEFAULT_PC_ADDRESS: Address = 0x200;
/// The number of bytes in an instruction.
pub const INSTRUCTION_BYTES: Address = 2;
/// The index of the flag register (a.k.a. VF).
pub const FLAG_REGISTER_INDEX: usize = 0xF;
/// The number of HP48 registers
pub const NUM_HP48_REGISTERS: usize = 8;

/// Data structure encapsulating CPU state at a moment in time.
pub struct Cpu {
    /// The 16 8-bit registers
    pub v: [Register; NUM_REGISTERS],
    /// The 16-bit address register
    pub i: Address,
    /// The 16-bit program counter
    pub pc: Address,
    /// The 8-bit stack pointer
    pub sp: Register,
    /// The 16 item stack
    pub stack: [Address; MAX_NUM_STACK_ITEMS],
    /// The 8 HP48 flag registers
    pub hp48: [Register; NUM_HP48_REGISTERS],
}

impl Cpu {
    /// Initialize a new `Cpu` data structure with sane default values.
    ///
    /// # Examples
    ///
    /// When instantiating a new Chip8 cpu:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu::Cpu;
    /// let cpu = Cpu::new();
    /// ```
    pub fn new() -> Self {
        Self {
            v: [0; NUM_REGISTERS],
            i: 0,
            pc: DEFAULT_PC_ADDRESS,
            sp: 0,
            stack: [0; MAX_NUM_STACK_ITEMS],
            hp48: [0; NUM_HP48_REGISTERS],
        }
    }

    /// Attempts to push a new value on top of the machine stack. On success it
    /// increments the stack pointer. On failure, it returns an error value.
    ///
    /// # Examples
    ///
    /// Calling push has the effect of incrementing the stack pointer and
    /// setting the previous value in the stack, as follows:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu::Cpu;
    /// let mut cpu = Cpu::new();
    ///
    /// assert_eq!(0, cpu.sp);
    /// assert_eq!(0, cpu.stack[0]);
    ///
    /// cpu.push_stack(0x123).unwrap();
    ///
    /// assert_eq!(1, cpu.sp);
    /// assert_eq!(0x123, cpu.stack[0]);
    /// ```
    ///
    /// It returns an error if the maximum stack size is exceeded:
    ///
    /// ```should_panic
    /// # extern crate okto;
    /// # use okto::cpu::{Cpu, MAX_NUM_STACK_ITEMS};
    /// # let mut cpu = Cpu::new();
    /// for _ in 0..=MAX_NUM_STACK_ITEMS { cpu.push_stack(0x123).unwrap() }
    /// ```
    pub fn push_stack(&mut self, value: Address) -> OktoResult<()> {
        if self.sp as usize > MAX_NUM_STACK_ITEMS {
            return Err(OktoError::new(OktoErrorKind::StackOverflow));
        }

        self.stack[self.sp as usize] = value;
        self.sp += 1;

        Ok(())
    }

    /// Attempts to pop the machine stack. On success it decrements the stack
    /// pointer and returns the address value previously on the top of the
    /// stack. On failure it returns `None` indicating a stack underflow error.
    ///
    /// # Examples
    ///
    /// When used correctly the stack can be popped several times:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu::Cpu;
    /// # let mut cpu = Cpu::new();
    /// cpu.push_stack(0x123).unwrap();
    /// assert_eq!(1, cpu.sp);
    /// assert_eq!(Some(0x123), cpu.pop_stack());
    /// cpu.push_stack(0x456).unwrap();
    /// assert_eq!(1, cpu.sp);
    /// assert_eq!(Some(0x456), cpu.pop_stack());
    /// ```
    ///
    /// An unsuccessful pop returns a `None` value as follows:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu::Cpu;
    /// let mut cpu = Cpu::new();
    /// assert_eq!(None, cpu.pop_stack());
    /// ```
    pub fn pop_stack(&mut self) -> Option<Address> {
        if self.sp <= 0 {
            return None;
        }

        self.sp -= 1;
        Some(self.stack[self.sp as usize])
    }

    /// Set the value of the flag register.
    ///
    /// # Examples
    ///
    /// You can set and then check the value of the flag register as follows:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu;
    /// # let mut cpu = cpu::Cpu::new();
    /// assert_eq!(0, cpu.v[cpu::FLAG_REGISTER_INDEX]);
    /// cpu.set_flag_reg(0x1);
    /// assert_eq!(1, cpu.v[cpu::FLAG_REGISTER_INDEX]);
    /// ```
    pub fn set_flag_reg(&mut self, value: Register) {
        self.v[FLAG_REGISTER_INDEX] = value;
    }

    /// Skips the next instruction by incrementing the program counter past it.
    ///
    /// # Examples
    ///
    /// Skipping the next instruction should change the program counter as
    /// follows:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu;
    /// # let mut cpu = cpu::Cpu::new();
    /// assert_eq!(0x200, cpu.pc);
    /// cpu.skip_next_instr();
    /// assert_eq!(0x200 + cpu::INSTRUCTION_BYTES, cpu.pc);
    /// ```
    pub fn skip_next_instr(&mut self) {
        self.pc += INSTRUCTION_BYTES;
    }
}

/// Parts of an instruction value that can be retrieved independently.
pub trait InstructionParts {
    /// Returns the 4-bit x register value from an opcode.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu::{Instruction, InstructionParts};
    /// assert_eq!(0x3, (0x8345 as Instruction).vx());
    /// ```
    fn vx(&self) -> Register;

    /// Returns the 4-bit y register value from an opcode.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu::{Instruction, InstructionParts};
    /// assert_eq!(0x4, (0x8345 as Instruction).vy());
    /// ```
    fn vy(&self) -> Register;

    /// Returns the 16-bit address value from an opcode.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu::{Instruction, InstructionParts};
    /// assert_eq!(0x345, (0x8345 as Instruction).addr());
    /// ```
    fn addr(&self) -> Address;

    /// Returns the 8-bit immediate value from an opcode.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu::{Instruction, InstructionParts};
    /// assert_eq!(0x45, (0x8345 as Instruction).imm());
    /// ```
    fn imm(&self) -> Immediate;

    /// Returns the 4-bit nibble value from an opcode.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu::{Instruction, InstructionParts};
    /// assert_eq!(0x5, (0x8345 as Instruction).nib());
    /// ```
    fn nib(&self) -> Nibble;
}

/// Allow access to instruction parts functions for `Instruction`.
impl InstructionParts for Instruction {
    fn vx(&self) -> Register {
        ((self & 0x0F00) >> 8) as Register
    }

    fn vy(&self) -> Register {
        ((self & 0x00F0) >> 4) as Register
    }

    fn addr(&self) -> Address {
        (self & 0x0FFF) as Address
    }

    fn imm(&self) -> Immediate {
        (self & 0x00FF) as Immediate
    }

    fn nib(&self) -> Nibble {
        (self & 0x000F) as Nibble
    }
}

/// Enumeration of Chip8 and SuperChip8 CPU operations
#[derive(Debug, PartialEq)]
pub enum Operation {
    // Chip8 Opcodes
    Cls,
    Ret,
    Sys(Address),
    Jump(Address),
    JumpAddrPlusV0(Address),
    Call(Address),
    SkipEqImm(Register, Immediate),
    SkipEqReg(Register, Register),
    SkipNeqImm(Register, Immediate),
    SkipNeqReg(Register, Register),
    LoadImm(Register, Immediate),
    LoadReg(Register, Register),
    LoadAddr(Address),
    LoadAddrDigit(Register),
    LoadRegDelay(Register),
    LoadDelayReg(Register),
    LoadSoundReg(Register),
    AddImm(Register, Immediate),
    AddReg(Register, Register),
    AddAddrReg(Register),
    Sub(Register, Register),
    SubNeg(Register, Register),
    Or(Register, Register),
    And(Register, Register),
    Xor(Register, Register),
    Shr(Register),
    Shl(Register),
    RandModImm(Register, Immediate),
    Draw(Register, Register, Nibble),
    SkipKey(Register),
    SkipNotKey(Register),
    WaitKey(Register),
    MemStoreBcd(Register),
    MemStoreRegs(Register),
    MemLoadRegs(Register),

    // SuperChip8 Opcodes
    Scd(Nibble),
    Scr,
    Scl,
    Exit,
    Low,
    High,
    LoadAddrBigDigit(Register),
    RplStoreRegs(Register),
    RplLoadRegs(Register),
}

impl Operation {
    /// Attempts to return the `Operation` corresponding to the given
    /// instruction bytes. If successful, it will return the operation along
    /// with its parameters. Otherwise, it returns `None`.
    ///
    /// # Examples
    ///
    /// For example a draw operation would produce the following:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu::{Instruction, Operation};
    /// assert_eq!(
    ///   Some(Operation::Draw(4, 5, 3)),
    ///   Operation::from_instruction(&0xD453)
    /// );
    /// ```
    ///
    /// An invalid operation would simply result in a `None` return value, as
    /// follows:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu::{Instruction, Operation};
    /// assert_eq!(None, Operation::from_instruction(&0xF178));
    /// ```
    pub fn from_instruction(instruction: &Instruction) -> Option<Operation> {
        match instruction & 0xF000 {
            0x0000 => match instruction & 0x0FFF {
                0x00E0 => Some(Operation::Cls),
                0x00EE => Some(Operation::Ret),
                0x00FB => Some(Operation::Scr),
                0x00FC => Some(Operation::Scl),
                0x00FD => Some(Operation::Exit),
                0x00FE => Some(Operation::Low),
                0x00FF => Some(Operation::High),
                _ => match instruction & 0x00F0 {
                    0x00C0 => Some(Operation::Scd(instruction.nib())),
                    _ => Some(Operation::Sys(instruction.addr())),
                },
            },
            0x1000 => Some(Operation::Jump(instruction.addr())),
            0x2000 => Some(Operation::Call(instruction.addr())),
            0x3000 => Some(Operation::SkipEqImm(instruction.vx(), instruction.imm())),
            0x4000 => Some(Operation::SkipNeqImm(instruction.vx(), instruction.imm())),
            0x5000 => match instruction & 0x000F {
                0x0000 => Some(Operation::SkipEqReg(instruction.vx(), instruction.vy())),
                _ => None,
            },
            0x6000 => Some(Operation::LoadImm(instruction.vx(), instruction.imm())),
            0x7000 => Some(Operation::AddImm(instruction.vx(), instruction.imm())),
            0x8000 => match instruction & 0x000F {
                0x0000 => Some(Operation::LoadReg(instruction.vx(), instruction.vy())),
                0x0001 => Some(Operation::Or(instruction.vx(), instruction.vy())),
                0x0002 => Some(Operation::And(instruction.vx(), instruction.vy())),
                0x0003 => Some(Operation::Xor(instruction.vx(), instruction.vy())),
                0x0004 => Some(Operation::AddReg(instruction.vx(), instruction.vy())),
                0x0005 => Some(Operation::Sub(instruction.vx(), instruction.vy())),
                0x0006 => Some(Operation::Shr(instruction.vx())),
                0x0007 => Some(Operation::SubNeg(instruction.vx(), instruction.vy())),
                0x000E => Some(Operation::Shl(instruction.vx())),
                _ => None,
            },
            0x9000 => match instruction & 0x000F {
                0x0000 => Some(Operation::SkipNeqReg(instruction.vx(), instruction.vy())),
                _ => None,
            },
            0xA000 => Some(Operation::LoadAddr(instruction.addr())),
            0xB000 => Some(Operation::JumpAddrPlusV0(instruction.addr())),
            0xC000 => Some(Operation::RandModImm(instruction.vx(), instruction.imm())),
            0xD000 => Some(Operation::Draw(
                instruction.vx(),
                instruction.vy(),
                instruction.nib(),
            )),
            0xE000 => match instruction & 0x00FF {
                0x009E => Some(Operation::SkipKey(instruction.vx())),
                0x00A1 => Some(Operation::SkipNotKey(instruction.vx())),
                _ => None,
            },
            0xF000 => match instruction & 0x00FF {
                0x0007 => Some(Operation::LoadRegDelay(instruction.vx())),
                0x000A => Some(Operation::WaitKey(instruction.vx())),
                0x0015 => Some(Operation::LoadDelayReg(instruction.vx())),
                0x0018 => Some(Operation::LoadSoundReg(instruction.vx())),
                0x001E => Some(Operation::AddAddrReg(instruction.vx())),
                0x0029 => Some(Operation::LoadAddrDigit(instruction.vx())),
                0x0030 => Some(Operation::LoadAddrBigDigit(instruction.vx())),
                0x0033 => Some(Operation::MemStoreBcd(instruction.vx())),
                0x0055 => Some(Operation::MemStoreRegs(instruction.vx())),
                0x0065 => Some(Operation::MemLoadRegs(instruction.vx())),
                0x0075 => Some(Operation::RplStoreRegs(instruction.vx())),
                0x0085 => Some(Operation::RplLoadRegs(instruction.vx())),
                _ => None,
            },
            _ => None,
        }
    }
}
