# Breakpoints

## How do they work

The way debuggers set breakpoints:

```s
0x12 | 0x34 | 0x56 | 0x78 ...
```

We take our memory. If we have a breakpoint at some address `x`, we perform a destructive action on the binary.

The steps:

- Go to address `x`
- Read the instruction currently there
- Overwrite it with a breakpoint instruction
- Return the old instruction

When a breakpoint is triggered:

- Stop execution
- Patch the old instruction in place of the instruction we just destroyed (it must be saved)
- Execute the instruction
- Stop at the next instruction
- Patch the breakpoint instruction back into the binary
