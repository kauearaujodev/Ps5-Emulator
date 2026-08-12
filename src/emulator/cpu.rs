use super::memory::VirtualMemory;

pub struct Cpu {
    pub registers: [u64; 16],

    pub pc: u64,

    pub halted: bool,

    pub instructions_executed: u64,
}

impl Cpu {

    pub fn new() -> Self {
        Self {
            registers: [0; 16],

            pc: 0,

            halted: false,

            instructions_executed: 0,
        }
    }

    pub fn reset(&mut self) {

        self.registers = [0; 16];

        self.pc = 0;

        self.halted = false;

        self.instructions_executed = 0;
    }

    pub fn step(
        &mut self,
        memory: &mut VirtualMemory,
    ) -> Result<(), String> {

        if self.halted {
            return Ok(());
        }

        let opcode = memory.read8(self.pc)?;

        match opcode {

            // LOAD R0, valor
            0x01 => {

                let value =
                    memory.read8(self.pc + 1)? as u64;

                self.registers[0] = value;

                self.pc += 2;
            }

            // LOAD R1, valor
            0x02 => {

                let value =
                    memory.read8(self.pc + 1)? as u64;

                self.registers[1] = value;

                self.pc += 2;
            }

            // ADD R2 = R0 + R1
            0x10 => {

                self.registers[2] =
                    self.registers[0]
                    .wrapping_add(self.registers[1]);

                self.pc += 1;
            }

            // SUB R2 = R0 - R1
            0x11 => {

                self.registers[2] =
                    self.registers[0]
                    .wrapping_sub(self.registers[1]);

                self.pc += 1;
            }

            // HALT
            0xFF => {

                self.halted = true;

                self.pc += 1;
            }

            _ => {

                return Err(format!(
                    "Opcode desconhecido: 0x{:02X} no PC 0x{:X}",
                    opcode,
                    self.pc
                ));
            }
        }

        self.instructions_executed += 1;

        Ok(())
    }

    pub fn run(
        &mut self,
        memory: &mut VirtualMemory,
        max_instructions: u64,
    ) -> Result<(), String> {

        while !self.halted
            && self.instructions_executed < max_instructions
        {
            self.step(memory)?;
        }

        Ok(())
    }
                           }
