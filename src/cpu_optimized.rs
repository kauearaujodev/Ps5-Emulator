//! CPU Otimizada com Cache, Pipeline e JIT
//! Performance máxima para emulação PS5

use super::memory::VirtualMemory;
use rayon::prelude::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

// ============================================
// CONSTANTES DE PERFORMANCE
// ============================================
pub const NUM_CORES: usize = 8;
pub const CACHE_SIZE: usize = 4096;
pub const PIPELINE_DEPTH: usize = 5;
pub const BRANCH_PREDICTOR_SIZE: usize = 1024;

// ============================================
// CACHE DE INSTRUÇÕES
// ============================================
#[derive(Clone, Debug)]
pub struct InstructionCache {
    pub cache: HashMap<u64, CachedInstruction>,
    pub hits: u64,
    pub misses: u64,
    pub max_size: usize,
}

#[derive(Clone, Debug)]
pub struct CachedInstruction {
    pub opcode: u8,
    pub size: u8,
    pub decoded: DecodedInstruction,
    pub last_used: Instant,
}

#[derive(Clone, Debug)]
pub struct DecodedInstruction {
    pub op_type: OpType,
    pub operands: Vec<Operand>,
    pub flags: u8,
    pub cycles: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OpType {
    Load, Store, Add, Sub, Mul, Div,
    And, Or, Xor, Shl, Shr,
    Jump, Call, Ret, Cmp, Halt,
    Mov, Push, Pop, Nop,
    Simd, Float,
}

#[derive(Clone, Debug)]
pub struct Operand {
    pub reg: Option<usize>,
    pub imm: Option<u64>,
    pub addr: Option<u64>,
    pub size: u8,
}

impl InstructionCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::with_capacity(CACHE_SIZE),
            hits: 0,
            misses: 0,
            max_size: CACHE_SIZE,
        }
    }

    pub fn get(&mut self, addr: u64) -> Option<CachedInstruction> {
        if let Some(inst) = self.cache.get(&addr) {
            self.hits += 1;
            Some(inst.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, addr: u64, inst: CachedInstruction) {
        if self.cache.len() >= self.max_size {
            // LRU - remove o mais antigo
            if let Some(oldest) = self.cache.iter()
                .min_by_key(|(_, v)| v.last_used)
                .map(|(k, _)| *k)
            {
                self.cache.remove(&oldest);
            }
        }
        self.cache.insert(addr, inst);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
    }

    pub fn hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f32 / total as f32 * 100.0 }
    }
}

// ============================================
// BRANCH PREDICTOR
// ============================================
#[derive(Clone, Debug)]
pub struct BranchPredictor {
    pub history: [u8; BRANCH_PREDICTOR_SIZE],
    pub predictions: u64,
    pub correct: u64,
    pub incorrect: u64,
}

impl BranchPredictor {
    pub fn new() -> Self {
        Self {
            history: [0; BRANCH_PREDICTOR_SIZE],
            predictions: 0,
            correct: 0,
            incorrect: 0,
        }
    }

    pub fn predict(&mut self, addr: u64) -> bool {
        self.predictions += 1;
        let idx = (addr & (BRANCH_PREDICTOR_SIZE as u64 - 1)) as usize;
        self.history[idx] >= 2 // 2-bit predictor
    }

    pub fn update(&mut self, addr: u64, taken: bool) {
        let idx = (addr & (BRANCH_PREDICTOR_SIZE as u64 - 1)) as usize;
        
        if taken {
            if self.history[idx] < 3 { self.history[idx] += 1; }
        } else {
            if self.history[idx] > 0 { self.history[idx] -= 1; }
        }
        
        // Verifica se a predição estava correta
        let predicted = self.history[idx] >= 2;
        if predicted == taken {
            self.correct += 1;
        } else {
            self.incorrect += 1;
        }
    }

    pub fn accuracy(&self) -> f32 {
        let total = self.correct + self.incorrect;
        if total == 0 { 0.0 } else { self.correct as f32 / total as f32 * 100.0 }
    }
}

// ============================================
// PIPELINE DE INSTRUÇÕES
// ============================================
#[derive(Clone, Debug)]
pub struct PipelineStage {
    pub instruction: Option<PipelineInstruction>,
    pub cycles: u8,
    pub busy: bool,
}

#[derive(Clone, Debug)]
pub struct PipelineInstruction {
    pub addr: u64,
    pub opcode: u8,
    pub decoded: DecodedInstruction,
    pub stage: PipelineStageType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PipelineStageType {
    Fetch,
    Decode,
    Execute,
    Memory,
    Writeback,
}

pub struct Pipeline {
    pub stages: [PipelineStage; PIPELINE_DEPTH],
    pub stalled: bool,
    pub stall_cycles: u8,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            stages: [
                PipelineStage { instruction: None, cycles: 0, busy: false },
                PipelineStage { instruction: None, cycles: 0, busy: false },
                PipelineStage { instruction: None, cycles: 0, busy: false },
                PipelineStage { instruction: None, cycles: 0, busy: false },
                PipelineStage { instruction: None, cycles: 0, busy: false },
            ],
            stalled: false,
            stall_cycles: 0,
        }
    }

    pub fn step(&mut self) -> bool {
        if self.stalled {
            self.stall_cycles -= 1;
            if self.stall_cycles == 0 {
                self.stalled = false;
            }
            return false;
        }

        // Move instruções pela pipeline
        let mut prev_inst: Option<PipelineInstruction> = None;
        
        for i in 0..PIPELINE_DEPTH {
            if self.stages[i].busy {
                if i == PIPELINE_DEPTH - 1 {
                    // Writeback - instrução completa
                    self.stages[i].busy = false;
                    self.stages[i].instruction = None;
                } else {
                    // Move para próximo estágio
                    let inst = self.stages[i].instruction.take();
                    self.stages[i].busy = false;
                    prev_inst = inst;
                }
            }
            
            if let Some(inst) = prev_inst.take() {
                self.stages[i].instruction = Some(inst);
                self.stages[i].busy = true;
                self.stages[i].cycles = 1;
            }
        }

        true
    }

    pub fn issue(&mut self, inst: PipelineInstruction) -> bool {
        // Tenta inserir na primeira etapa (Fetch)
        if !self.stages[0].busy {
            self.stages[0].instruction = Some(inst);
            self.stages[0].busy = true;
            self.stages[0].cycles = 1;
            true
        } else {
            self.stalled = true;
            self.stall_cycles = 1;
            false
        }
    }
}

// ============================================
// THREAD POOL
// ============================================
pub struct ThreadPool {
    pub num_threads: usize,
    pub running: bool,
}

impl ThreadPool {
    pub fn new() -> Self {
        let num_threads = rayon::current_num_threads();
        Self {
            num_threads,
            running: false,
        }
    }

    pub fn execute_parallel<F>(&self, work: F) 
    where F: Fn() + Send + Sync + 'static {
        rayon::spawn(work);
    }

    pub fn wait(&self) {
        rayon::global_pool().scope(|_s| {});
    }
}

// ============================================
// JIT COMPILER (Just-In-Time)
// ============================================
pub struct JitCompiler {
    pub enabled: bool,
    pub compiled: HashMap<u64, JitBlock>,
    pub hit_count: u64,
    pub miss_count: u64,
}

#[derive(Clone, Debug)]
pub struct JitBlock {
    pub start_addr: u64,
    pub end_addr: u64,
    pub instructions: Vec<DecodedInstruction>,
    pub native_code: Vec<u8>,
    pub execution_count: u64,
}

impl JitCompiler {
    pub fn new() -> Self {
        Self {
            enabled: true,
            compiled: HashMap::new(),
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub fn compile(&mut self, addr: u64, instructions: Vec<DecodedInstruction>) -> JitBlock {
        let block = JitBlock {
            start_addr: addr,
            end_addr: addr + instructions.len() as u64,
            instructions: instructions.clone(),
            native_code: self.generate_native_code(&instructions),
            execution_count: 0,
        };
        
        self.compiled.insert(addr, block.clone());
        block
    }

    pub fn get(&mut self, addr: u64) -> Option<&JitBlock> {
        if let Some(block) = self.compiled.get(&addr) {
            self.hit_count += 1;
            Some(block)
        } else {
            self.miss_count += 1;
            None
        }
    }

    pub fn generate_native_code(&self, instructions: &[DecodedInstruction]) -> Vec<u8> {
        // Simula geração de código nativo
        // Em um emulador real, isso seria código de máquina x86_64
        let mut code = Vec::new();
        
        for inst in instructions {
            // Representação simplificada
            code.push(inst.op_type as u8);
            code.push(inst.cycles);
            code.push(inst.flags);
            
            // Adiciona operandos
            for op in &inst.operands {
                if let Some(reg) = op.reg {
                    code.push(reg as u8);
                }
                if let Some(imm) = op.imm {
                    code.extend_from_slice(&imm.to_le_bytes());
                }
                if let Some(addr) = op.addr {
                    code.extend_from_slice(&addr.to_le_bytes());
                }
            }
        }
        
        code
    }

    pub fn hit_rate(&self) -> f32 {
        let total = self.hit_count + self.miss_count;
        if total == 0 { 0.0 } else { self.hit_count as f32 / total as f32 * 100.0 }
    }
}

// ============================================
// CPU CORE OTIMIZADA
// ============================================
pub struct CpuCoreOptimized {
    // Registradores
    pub registers: [u64; 16],
    pub rip: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub rflags: u64,
    
    // Core info
    pub core_id: usize,
    pub status: CoreStatus,
    pub frequency_mhz: u32,
    
    // Performance
    pub instructions_executed: u64,
    pub cycles: u64,
    pub cache: InstructionCache,
    pub branch_predictor: BranchPredictor,
    pub pipeline: Pipeline,
    pub jit: JitCompiler,
    
    // Stats
    pub ipc: f32, // Instructions Per Cycle
    pub last_cycle_time: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CoreStatus {
    Active,
    Idle,
    Halted,
    PowerSaving,
}

impl CpuCoreOptimized {
    pub fn new(core_id: usize) -> Self {
        Self {
            registers: [0; 16],
            rip: 0,
            rsp: 0xFFFF_FFFF_FFFF_0000,
            rbp: 0,
            rflags: 0,
            core_id,
            status: CoreStatus::Active,
            frequency_mhz: 3500,
            instructions_executed: 0,
            cycles: 0,
            cache: InstructionCache::new(),
            branch_predictor: BranchPredictor::new(),
            pipeline: Pipeline::new(),
            jit: JitCompiler::new(),
            ipc: 0.0,
            last_cycle_time: Instant::now(),
        }
    }

    pub fn step(&mut self, memory: &mut VirtualMemory) -> Result<(), String> {
        if self.status == CoreStatus::Halted {
            return Ok(());
        }

        self.cycles += 1;

        // ============================================
        // 1. CHECA CACHE DE INSTRUÇÕES
        // ============================================
        let opcode = if let Some(cached) = self.cache.get(self.rip) {
            cached.opcode
        } else {
            // Cache miss - busca na memória
            let opcode = memory.read8(self.rip)?;
            
            // Decoda e cacheia
            let decoded = self.decode_instruction(opcode, memory)?;
            let cached = CachedInstruction {
                opcode,
                size: 1,
                decoded,
                last_used: Instant::now(),
            };
            self.cache.insert(self.rip, cached);
            opcode
        };

        // ============================================
        // 2. JIT - COMPILAÇÃO
        // ============================================
        if self.jit.enabled {
            if let Some(block) = self.jit.get(self.rip) {
                // Usa código compilado
                self.execute_jit_block(block, memory)?;
                return Ok(());
            } else {
                // Compila bloco
                let instructions = self.collect_basic_block(self.rip, memory)?;
                let block = self.jit.compile(self.rip, instructions);
                self.execute_jit_block(&block, memory)?;
                return Ok(());
            }
        }

        // ============================================
        // 3. EXECUTA INSTRUÇÃO (Fallback)
        // ============================================
        self.execute_instruction(opcode, memory)?;
        self.instructions_executed += 1;

        // ============================================
        // 4. ATUALIZA IPC
        // ============================================
        let elapsed = self.last_cycle_time.elapsed();
        if elapsed.as_secs_f32() > 0.001 {
            self.ipc = self.instructions_executed as f32 / self.cycles as f32;
        }

        Ok(())
    }

    fn decode_instruction(&self, opcode: u8, memory: &VirtualMemory) -> Result<DecodedInstruction, String> {
        // Decodificação otimizada
        let decoded = match opcode {
            // MOV
            0x48 => {
                let next = memory.read8(self.rip + 1)?;
                if next >= 0xB8 && next <= 0xBF {
                    DecodedInstruction {
                        op_type: OpType::Mov,
                        operands: vec![
                            Operand { reg: Some((next - 0xB8) as usize), imm: None, addr: None, size: 8 },
                            Operand { reg: None, imm: Some(memory.read64(self.rip + 2)?), addr: None, size: 8 },
                        ],
                        flags: 0,
                        cycles: 1,
                    }
                } else {
                    DecodedInstruction {
                        op_type: OpType::Mov,
                        operands: vec![],
                        flags: 0,
                        cycles: 1,
                    }
                }
            },
            
            // ADD
            0x10 | 0x81 => {
                DecodedInstruction {
                    op_type: OpType::Add,
                    operands: vec![
                        Operand { reg: Some(0), imm: None, addr: None, size: 8 },
                        Operand { reg: Some(1), imm: None, addr: None, size: 8 },
                    ],
                    flags: 0,
                    cycles: 1,
                }
            },
            
            // SUB
            0x11 | 0x83 => {
                DecodedInstruction {
                    op_type: OpType::Sub,
                    operands: vec![
                        Operand { reg: Some(0), imm: None, addr: None, size: 8 },
                        Operand { reg: Some(1), imm: None, addr: None, size: 8 },
                    ],
                    flags: 0,
                    cycles: 1,
                }
            },
            
            // HALT
            0xF4 => {
                DecodedInstruction {
                    op_type: OpType::Halt,
                    operands: vec![],
                    flags: 0,
                    cycles: 1,
                }
            },
            
            // Jump
            0xEB | 0xE9 | 0x74 | 0x75 | 0x7C | 0x7D => {
                DecodedInstruction {
                    op_type: OpType::Jump,
                    operands: vec![
                        Operand { reg: None, imm: Some(memory.read8(self.rip + 1)? as u64), addr: None, size: 1 },
                    ],
                    flags: 0,
                    cycles: 1,
                }
            },
            
            // Call
            0xE8 => {
                DecodedInstruction {
                    op_type: OpType::Call,
                    operands: vec![
                        Operand { reg: None, imm: Some(memory.read32(self.rip + 1)? as u64), addr: None, size: 4 },
                    ],
                    flags: 0,
                    cycles: 2,
                }
            },
            
            // Ret
            0xC3 => {
                DecodedInstruction {
                    op_type: OpType::Ret,
                    operands: vec![],
                    flags: 0,
                    cycles: 2,
                }
            },
            
            // NOP
            0x90 => {
                DecodedInstruction {
                    op_type: OpType::Nop,
                    operands: vec![],
                    flags: 0,
                    cycles: 1,
                }
            },
            
            _ => {
                return Err(format!("Opcode desconhecido: 0x{:02X}", opcode));
            }
        };
        
        Ok(decoded)
    }

    fn collect_basic_block(&self, start_addr: u64, memory: &VirtualMemory) -> Result<Vec<DecodedInstruction>, String> {
        let mut instructions = Vec::new();
        let mut addr = start_addr;
        
        for _ in 0..32 { // Max 32 instruções por bloco
            let opcode = memory.read8(addr)?;
            let decoded = self.decode_instruction(opcode, memory)?;
            instructions.push(decoded.clone());
            
            addr += 1;
            
            // Para ao encontrar um salto
            if matches!(decoded.op_type, OpType::Jump | OpType::Call | OpType::Ret | OpType::Halt) {
                break;
            }
        }
        
        Ok(instructions)
    }

    fn execute_jit_block(&mut self, block: &JitBlock, memory: &mut VirtualMemory) -> Result<(), String> {
        // Executa código compilado (simulado)
        for (i, inst) in block.instructions.iter().enumerate() {
            let addr = block.start_addr + i as u64;
            
            // Tenta decodificar da cache
            if let Some(cached) = self.cache.get(addr) {
                self.execute_instruction(cached.opcode, memory)?;
            } else {
                // Fallback
                let opcode = memory.read8(addr)?;
                self.execute_instruction(opcode, memory)?;
            }
            
            self.instructions_executed += 1;
            self.cycles += 1;
        }
        
        // Atualiza RIP para próximo bloco
        self.rip = block.end_addr;
        
        Ok(())
    }

    fn execute_instruction(&mut self, opcode: u8, memory: &mut VirtualMemory) -> Result<(), String> {
        match opcode {
            // MOV
            0x48 => {
                let next = memory.read8(self.rip + 1)?;
                if next >= 0xB8 && next <= 0xBF {
                    let reg = (next - 0xB8) as usize;
                    let value = memory.read64(self.rip + 2)?;
                    self.registers[reg] = value;
                    self.rip += 10;
                } else {
                    self.rip += 1;
                }
            }
            
            // ADD R0, R1
            0x10 => {
                self.registers[2] = self.registers[0].wrapping_add(self.registers[1]);
                self.rip += 1;
            }
            
            // SUB R0, R1
            0x11 => {
                self.registers[2] = self.registers[0].wrapping_sub(self.registers[1]);
                self.rip += 1;
            }
            
            // CMP
            0x3D => {
                let imm = memory.read32(self.rip + 1)? as u64;
                let result = self.registers[0].wrapping_sub(imm);
                self.rflags = if result == 0 { 1 } else { 0 };
                self.rip += 5;
            }
            
            // JE (Jump if Equal)
            0x74 => {
                let offset = memory.read8(self.rip + 1)? as i8;
                if self.rflags & 0x40 != 0 {
                    self.rip = self.rip.wrapping_add(offset as u64 + 2);
                    self.branch_predictor.update(self.rip, true);
                } else {
                    self.rip += 2;
                    self.branch_predictor.update(self.rip, false);
                }
            }
            
            // JMP
            0xEB | 0xE9 => {
                let offset = memory.read8(self.rip + 1)? as i8;
                self.rip = self.rip.wrapping_add(offset as u64 + 2);
                self.branch_predictor.update(self.rip, true);
            }
            
            // CALL
            0xE8 => {
                let offset = memory.read32(self.rip + 1)? as i32;
                self.rsp = self.rsp.wrapping_sub(8);
                memory.write64(self.rsp, self.rip + 5)?;
                self.rip = self.rip.wrapping_add(offset as u64 + 5);
            }
            
            // RET
            0xC3 => {
                let return_addr = memory.read64(self.rsp)?;
                self.rsp = self.rsp.wrapping_add(8);
                self.rip = return_addr;
            }
            
            // HALT
            0xF4 => {
                self.status = CoreStatus::Halted;
                self.rip += 1;
            }
            
            _ => {
                self.rip += 1;
            }
        }
        
        Ok(())
    }

    pub fn reset(&mut self) {
        self.registers = [0; 16];
        self.rip = 0;
        self.rsp = 0xFFFF_FFFF_FFFF_0000;
        self.rbp = 0;
        self.rflags = 0;
        self.status = CoreStatus::Active;
        self.instructions_executed = 0;
        self.cycles = 0;
        self.cache.clear();
        self.jit.compiled.clear();
        self.ipc = 0.0;
    }

    pub fn get_stats(&self) -> String {
        format!(
            "Core {}: {} instr, {} cycles, IPC: {:.2}, Cache: {:.1}%, Branch: {:.1}%, JIT: {:.1}%",
            self.core_id,
            self.instructions_executed,
            self.cycles,
            self.ipc,
            self.cache.hit_rate(),
            self.branch_predictor.accuracy(),
            self.jit.hit_rate()
        )
    }
}

// ============================================
// CPU PS5 OTIMIZADA
// ============================================
pub struct Ps5CpuOptimized {
    pub cores: [CpuCoreOptimized; NUM_CORES],
    pub total_instructions: u64,
    pub total_cycles: u64,
    pub thread_pool: ThreadPool,
    pub start_time: Instant,
}

impl Ps5CpuOptimized {
    pub fn new() -> Self {
        let cores = [
            CpuCoreOptimized::new(0),
            CpuCoreOptimized::new(1),
            CpuCoreOptimized::new(2),
            CpuCoreOptimized::new(3),
            CpuCoreOptimized::new(4),
            CpuCoreOptimized::new(5),
            CpuCoreOptimized::new(6),
            CpuCoreOptimized::new(7),
        ];
        
        Self {
            cores,
            total_instructions: 0,
            total_cycles: 0,
            thread_pool: ThreadPool::new(),
            start_time: Instant::now(),
        }
    }

    pub fn step_all(&mut self, memory: &mut VirtualMemory) -> Result<(), String> {
        let start = Instant::now();
        
        // Executa todos os cores em paralelo
        let cores = &mut self.cores;
        let memory_ref = memory;
        
        // Usa Rayon para paralelizar
        cores.par_iter_mut().for_each(|core| {
            let _ = core.step(memory_ref);
        });
        
        // Atualiza estatísticas
        for core in cores.iter() {
            self.total_instructions += core.instructions_executed;
            self.total_cycles += core.cycles;
        }
        
        Ok(())
    }

    pub fn step_core(&mut self, core_id: usize, memory: &mut VirtualMemory) -> Result<(), String> {
        if core_id >= NUM_CORES {
            return Err("Core ID inválido".to_string());
        }
        
        self.cores[core_id].step(memory)?;
        self.total_instructions += self.cores[core_id].instructions_executed;
        self.total_cycles += self.cores[core_id].cycles;
        
        Ok(())
    }

    pub fn run(&mut self, memory: &mut VirtualMemory, max_instructions: u64) -> Result<(), String> {
        let mut executed = 0;
        
        while executed < max_instructions {
            self.step_all(memory)?;
            
            // Verifica se todos estão haltados
            let all_halted = self.cores.iter().all(|c| c.status == CoreStatus::Halted);
            if all_halted {
                break;
            }
            
            executed += 1;
        }
        
        Ok(())
    }

    pub fn reset_all(&mut self) {
        for core in &mut self.cores {
            core.reset();
        }
        self.total_instructions = 0;
        self.total_cycles = 0;
        self.start_time = Instant::now();
    }

    pub fn get_stats(&self) -> String {
        let mut stats = String::new();
        stats.push_str("=== PS5 CPU PERFORMANCE STATS ===\n");
        stats.push_str(&format!("Total Instructions: {}\n", self.total_instructions));
        stats.push_str(&format!("Total Cycles: {}\n", self.total_cycles));
        
        let elapsed = self.start_time.elapsed();
        let ipc = self.total_instructions as f32 / self.total_cycles as f32;
        stats.push_str(&format!("IPC: {:.2}\n", ipc));
        stats.push_str(&format!("Time: {:.2}s\n", elapsed.as_secs_f32()));
        stats.push_str(&format!("Speed: {:.2} MIPS\n", 
            self.total_instructions as f32 / elapsed.as_secs_f32() / 1_000_000.0));
        
        stats.push_str("\n=== CORE DETAILS ===\n");
        for core in &self.cores {
            stats.push_str(&format!("{}\n", core.get_stats()));
        }
        
        stats
    }
}

// ============================================
// TESTES DE PERFORMANCE
// ============================================
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_performance() {
        let mut cache = InstructionCache::new();
        let addr = 0x1000;
        
        let inst = CachedInstruction {
            opcode: 0x90,
            size: 1,
            decoded: DecodedInstruction {
                op_type: OpType::Nop,
                operands: vec![],
                flags: 0,
                cycles: 1,
            },
            last_used: Instant::now(),
        };
        
        cache.insert(addr, inst.clone());
        let retrieved = cache.get(addr);
        assert!(retrieved.is_some());
        assert_eq!(cache.hits, 1);
        assert_eq!(cache.misses, 0);
    }
    
    #[test]
    fn test_branch_predictor() {
        let mut predictor = BranchPredictor::new();
        let addr = 0x1000;
        
        let predicted = predictor.predict(addr);
        assert_eq!(predicted, false); // Estado inicial
    }
    
    #[test]
    fn test_cpu_core_step() {
        let mut core = CpuCoreOptimized::new(0);
        let mut memory = VirtualMemory::new(1024 * 1024);
        
        // Escreve um NOP
        memory.write8(0x1000, 0x90).unwrap();
        core.rip = 0x1000;
        
        let result = core.step(&mut memory);
        assert!(result.is_ok());
        assert_eq!(core.instructions_executed, 1);
    }
      }
