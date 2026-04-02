@ submission_01.asm
@ Task: Add 10 and 20, store result in the peripheral, and exit.

.global _start
_start:
    @ 1. Perform Calculation
    MOV R1, #10
    MOV R2, #20
    ADD R3, R1, R2      @ R3 = 30 (0x1E)

    @ 2. Interaction with Hardware
    @ Grader will provide the symbol 'IO_BASE'
    LDR R0, =IO_BASE    
    STR R3, [R0]        @ Write 30 to the peripheral

    @ 3. Preparation for Exit
    MOV R0, #0          @ Set return code to 0 (Success)
    MOV R7, #1          @ Syscall 1 = Exit
    SVC 0               @ Trigger Supervisor Call
    