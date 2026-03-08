use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use emulator::{
    Emulator,
    cpu::Cpu,
    memory::{Bus, Endian},
};

/// Helper to set up an emulator with a specific block of machine code.
fn setup_emulator(code: &[u8]) -> Emulator {
    // Initialize with 64KB Code, 64KB SRAM, 0 External
    let mut bus = Bus::new(65536, 65536, 0);
    bus.load_code(code);

    let mut cpu = Cpu::new();
    // Initialize Stack Pointer to the top of SRAM
    cpu.set_sp(Bus::SRAM_BEGIN + 65536);

    Emulator::new(cpu, bus, Endian::Little)
}

// 1. ARITHMETIC BENCHMARK
// 0x00: ADD R0, R0, #1  ; E2800001
// 0x04: SUB R1, R1, #1  ; E2411001
// 0x08: MUL R2, R0, R1  ; E0020190
// 0x0C: B 0x00          ; EAFFFFFB (Branch back to 0x00)
const ARITHMETIC_LOOP: [u8; 16] = [
    0x01, 0x00, 0x80, 0xE2, 0x01, 0x10, 0x41, 0xE2, 0x90, 0x01, 0x02,
    0xE0, 0xFB, 0xFF, 0xFF, 0xEA,
];

fn bench_arithmetic(c: &mut Criterion) {
    let mut emu = setup_emulator(&ARITHMETIC_LOOP);
    c.bench_function("execute_arithmetic_1000_steps", |b| {
        b.iter(|| {
            // We step 1,000 times per iteration to average out measurement overhead.
            for _ in 0..1000 {
                black_box(emu.step().unwrap());
            }
        });
    });
}

// 2. MEMORY BENCHMARK
// 0x00: LDR R0,[R1]     ; E5910000
// 0x04: STR R0, [R2]    ; E5820000
// 0x08: B 0x00          ; EAFFFFFC (Branch back to 0x00)
const MEMORY_LOOP: [u8; 12] = [
    0x00, 0x00, 0x91, 0xE5, 0x00, 0x00, 0x82, 0xE5, 0xFC, 0xFF, 0xFF, 0xEA,
];

fn bench_memory(c: &mut Criterion) {
    let mut emu = setup_emulator(&MEMORY_LOOP);

    // Set R1 and R2 to valid, aligned SRAM addresses
    emu.cpu.set_register(1, Bus::SRAM_BEGIN);
    emu.cpu.set_register(2, Bus::SRAM_BEGIN + 4);

    c.bench_function("execute_memory_1000_steps", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(emu.step().unwrap());
            }
        });
    });
}

// 3. BRANCHING / CONTROL FLOW BENCHMARK
// Simulates a countdown loop
// 0x00: MOV R0, #100    ; E3A00064
// 0x04: SUBS R0, R0, #1 ; E2500001
// 0x08: BNE 0x04        ; 1AFFFFFD (Branch back to 0x04 if not zero)
// 0x0C: B 0x00          ; EAFFFFFB (Reset loop back to 0x00)
const BRANCH_LOOP: [u8; 16] = [
    0x64, 0x00, 0xA0, 0xE3, 0x01, 0x00, 0x50, 0xE2, 0xFD, 0xFF, 0xFF,
    0x1A, 0xFB, 0xFF, 0xFF, 0xEA,
];

fn bench_branching(c: &mut Criterion) {
    let mut emu = setup_emulator(&BRANCH_LOOP);
    c.bench_function("execute_branching_1000_steps", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(emu.step().unwrap());
            }
        });
    });
}

// 4. MIXED WORKLOAD BENCHMARK (Function calls, stack, math)
// 0x00: MOV R0, #5      ; E3A00005
// 0x04: BL 0x0C         ; EB000000 (Call function at 0x0C)
// 0x08: B 0x00          ; EAFFFFFC (Loop back to start)
// function at 0x0C:
// 0x0C: PUSH {R0, LR}   ; E92D4001
// 0x10: ADD R0, R0, #1  ; E2800001
// 0x14: POP {R0, PC}    ; E8BD8001
const MIXED_LOOP: [u8; 24] = [
    0x05, 0x00, 0xA0, 0xE3, 0x00, 0x00, 0x00, 0xEB, 0xFC, 0xFF, 0xFF,
    0xEA, 0x01, 0x40, 0x2D, 0xE9, 0x01, 0x00, 0x80, 0xE2, 0x01, 0x80,
    0xBD, 0xE8,
];

fn bench_mixed(c: &mut Criterion) {
    let mut emu = setup_emulator(&MIXED_LOOP);
    c.bench_function("execute_mixed_1000_steps", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(emu.step().unwrap());
            }
        });
    });
}

// Register the benchmarks
criterion_group!(
    benches,
    bench_arithmetic,
    bench_memory,
    bench_branching,
    bench_mixed
);
criterion_main!(benches);
