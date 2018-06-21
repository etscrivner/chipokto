//! Types and data structures representing the assembled Chip8 machine.
use super::{OktoError, OktoErrorKind, OktoResult};
use super::cpu;
use super::display;
use super::keyboard;
use super::memory;
use super::sound;
use super::timer;

use rand::prelude::*;

/// Data structure representing the complete machine.
pub struct Machine<F>
where F: FnMut() -> keyboard::WaitKeyResult<u8>
{
    /// CPU component of the machine
    pub cpu: cpu::Cpu,
    /// Delay countdown timer
    pub delay_timer: timer::DelayTimer,
    /// Video display
    pub display: display::Display,
    /// Keyboard component
    pub keyboard: keyboard::Keyboard<F>,
    /// Memory component of the machine
    pub memory: memory::Memory,
    /// Sound card, which is really a glorified timer
    pub sound: sound::Sound
}

impl<F> Machine<F>
where F: FnMut() -> keyboard::WaitKeyResult<u8> {
    /// Construct a new machine with all of its components.
    pub fn new(
        wait_key_callback: Box<F>
    ) -> Self {
        Self {
            cpu: cpu::Cpu::new(),
            delay_timer: timer::DelayTimer::new(),
            display: display::Display::new(),
            keyboard: keyboard::Keyboard::new(wait_key_callback),
            memory: memory::Memory::new(),
            sound: sound::Sound::new()
        }
    }

    /// Executes a single operation on the machine and return the resulting
    /// updated machine or an error if the operation failed.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu::Operation;
    /// # use okto::machine::Machine;
    /// # use okto::keyboard;
    /// # let mut machine = Machine::new(
    /// #   Box::new(keyboard::nop_wait_key_callback)
    /// # );
    /// assert_eq!(0, machine.cpu.v[0x3]);
    /// machine.execute(Operation::LoadImm(0x3, 0x25));
    /// assert_eq!(0x25, machine.cpu.v[0x3]);
    /// ```
    ///
    /// A failure may occur if, for example, a program tries to pop from an
    /// empty call stack like the following:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::cpu::Operation;
    /// # use okto::machine::Machine;
    /// # use okto::keyboard;
    /// # let mut machine = Machine::new(
    /// #   Box::new(keyboard::nop_wait_key_callback)
    /// # );
    /// let result = machine.execute(Operation::Ret);
    /// assert!(result.is_err());
    /// ```
    pub fn execute(&mut self, operation: cpu::Operation)
               -> OktoResult<&mut Self>
    {
        match operation {
            cpu::Operation::Cls => self.display.clear(),
            cpu::Operation::Ret => {
                if let Some(return_addr) = self.cpu.pop_stack() {
                    self.cpu.pc = return_addr;
                } else {
                    return Err(OktoError::new(OktoErrorKind::StackUnderflow));
                }
            },
            cpu::Operation::Sys(addr) => self.cpu.pc = addr,
            cpu::Operation::Jump(addr) => self.cpu.pc = addr,
            cpu::Operation::JumpAddrPlusV0(addr) => {
                self.cpu.pc = addr + (self.cpu.v[0] as u16)
            },
            cpu::Operation::Call(addr) => {
                let pc = self.cpu.pc;
                if let Err(error) = self.cpu.push_stack(pc) {
                    return Err(
                        OktoError::new(OktoErrorKind::Unknown(error.to_string()))
                    );
                } else {
                    self.cpu.pc = addr;
                }
            },
            cpu::Operation::SkipEqImm(vx, imm) => {
                if self.cpu.v[vx as usize] == imm {
                    self.cpu.skip_next_instr();
                }
            },
            cpu::Operation::SkipEqReg(vx, vy) => {
                if self.cpu.v[vx as usize] == self.cpu.v[vy as usize] {
                    self.cpu.skip_next_instr();
                }
            },
            cpu::Operation::SkipNeqImm(vx, imm) => {
                if self.cpu.v[vx as usize] != imm {
                    self.cpu.skip_next_instr();
                }
            },
            cpu::Operation::SkipNeqReg(vx, vy) => {
                if self.cpu.v[vx as usize] != self.cpu.v[vy as usize] {
                    self.cpu.skip_next_instr();
                }
            },
            cpu::Operation::LoadImm(vx, imm) => self.cpu.v[vx as usize] = imm,
            cpu::Operation::LoadReg(vx, vy) => {
                self.cpu.v[vx as usize] = self.cpu.v[vy as usize];
            },
            cpu::Operation::LoadAddr(addr) => {
                self.cpu.i = addr;
            },
            cpu::Operation::LoadAddrDigit(vx) => {
                let sprite_address = self.memory.sprite_address_for_digit(
                    self.cpu.v[vx as usize]
                );

                if let Some(digit_addr) = sprite_address {
                    self.cpu.i = digit_addr;
                } else {
                    return Err(
                        OktoError::new(OktoErrorKind::InvalidDigitSprite)
                    );
                }
            },
            cpu::Operation::LoadRegDelay(vx) => {
                self.cpu.v[vx as usize] = self.delay_timer.value;
            },
            cpu::Operation::LoadDelayReg(vx) => {
                self.delay_timer.value = self.cpu.v[vx as usize];
            },
            cpu::Operation::LoadSoundReg(vx) => {
                self.sound.timer = self.cpu.v[vx as usize];
            },
            cpu::Operation::AddImm(vx, imm) => {
                self.cpu.v[vx as usize] += imm;
            },
            cpu::Operation::AddReg(vx, vy) => {
                let (result, overflowed) =
                    self.cpu.v[vx as usize].overflowing_add(self.cpu.v[vy as usize]);

                // Set flag register to indicate whether or not the addition
                // overflowed the 8-bits available.
                self.cpu.set_flag_reg(if overflowed { 0x01 } else { 0x00 });
                self.cpu.v[vx as usize] = result;
            },
            cpu::Operation::AddAddrReg(vx) => {
                self.cpu.i += self.cpu.v[vx as usize] as cpu::Address;
            },
            cpu::Operation::Sub(vx, vy) => {
                let (result, overflowed) =
                    self.cpu.v[vx as usize].overflowing_sub(
                        self.cpu.v[vy as usize]
                    );

                self.cpu.set_flag_reg(if overflowed { 0x01 } else { 0x00 });
                self.cpu.v[vx as usize] = result;
            },
            cpu::Operation::SubNeg(vx, vy) => {
                let (result, overflowed) =
                    self.cpu.v[vy as usize].overflowing_sub(
                        self.cpu.v[vx as usize]
                    );

                self.cpu.set_flag_reg(if overflowed { 0x01 } else { 0x00 });
                self.cpu.v[vy as usize] = result;
            },
            cpu::Operation::Or(vx, vy) => {
                self.cpu.v[vx as usize] |= self.cpu.v[vy as usize];
            },
            cpu::Operation::And(vx, vy) => {
                self.cpu.v[vx as usize] &= self.cpu.v[vy as usize];
            },
            cpu::Operation::Xor(vx, vy) => {
                self.cpu.v[vx as usize] ^= self.cpu.v[vy as usize];
            },
            cpu::Operation::Shr(vx) => {
                let flag_value = self.cpu.v[vx as usize] & 0x1;
                self.cpu.set_flag_reg(flag_value);
                self.cpu.v[vx as usize] >>= 1;
            },
            cpu::Operation::Shl(vx) => {
                let flag_value = (self.cpu.v[vx as usize] & 0x80) >> 7;
                self.cpu.set_flag_reg(flag_value);
                self.cpu.v[vx as usize] <<= 1;
            },
            cpu::Operation::RandModImm(vx, imm) => {
                self.cpu.v[vx as usize] = random::<u8>() % imm;
            },
            cpu::Operation::Draw(vx, vy, size_bytes) => {
                let sprite_data = self.memory.read_bytes(
                    self.cpu.i, size_bytes as usize
                )?;

                let pixels_erased = self.display.draw(
                    self.cpu.v[vx as usize] as usize,
                    self.cpu.v[vy as usize] as usize,
                    size_bytes as usize,
                    sprite_data
                );

                self.cpu.set_flag_reg(if pixels_erased { 0x01 } else { 0x00 });
            },
            cpu::Operation::SkipKey(vx) => {
                let index = self.cpu.v[vx as usize] as usize;
                if self.keyboard.keys[index] == keyboard::KeyState::Pressed {
                    self.cpu.skip_next_instr();
                }
            },
            cpu::Operation::SkipNotKey(vx) => {
                let index = self.cpu.v[vx as usize] as usize;
                if self.keyboard.keys[index] == keyboard::KeyState::Released {
                    self.cpu.skip_next_instr();
                }
            },
            cpu::Operation::WaitKey(vx) => {
                match (self.keyboard.wait_key_callback)() {
                    Ok(value) => self.cpu.v[vx as usize] = value,
                    Err(err) => {
                        return Err(OktoError::new(OktoErrorKind::Unknown(err)))
                    }
                }
            },
            cpu::Operation::MemStoreBcd(vx) => {
                let addr = self.cpu.i as usize;
                let value = self.cpu.v[vx as usize];

                self.memory.data[addr] = value / 100;
                self.memory.data[addr + 1] = (value / 10) % 10;
                self.memory.data[addr + 2] = value % 10;
            },
            cpu::Operation::MemStoreRegs(vx) => {
                for index in 0..=vx {
                    self.memory.data[(self.cpu.i + index as u16) as usize] =
                        self.cpu.v[index as usize];
                }
            },
            cpu::Operation::MemLoadRegs(vx) => {
                for index in 0..=vx {
                    self.cpu.v[index as usize] =
                        self.memory.data[(self.cpu.i + index as u16) as usize];
                }
            },
            _ => return Err(OktoError::new(OktoErrorKind::InvalidOpcode))
        }

        Ok(self)
    }
}

#[test]
fn machine_execution() {
    let mut machine = Machine::new(Box::new(keyboard::nop_wait_key_callback));
    assert_eq!(0, machine.cpu.v[3]);
    machine.execute(cpu::Operation::LoadImm(3, 0x25)).unwrap();
    assert_eq!(0x25, machine.cpu.v[3]);

    // Call and return
    machine.execute(cpu::Operation::Call(0x234)).unwrap();
    machine.execute(cpu::Operation::Ret).unwrap();

    // BCD encoding
    machine.execute(cpu::Operation::LoadImm(0, 234)).unwrap();
    machine.execute(cpu::Operation::LoadAddr(0x200)).unwrap();
    machine.execute(cpu::Operation::MemStoreBcd(0)).unwrap();

    assert_eq!(0x02, machine.memory.data[0x200]);
    assert_eq!(0x03, machine.memory.data[0x201]);
    assert_eq!(0x04, machine.memory.data[0x202]);
}
