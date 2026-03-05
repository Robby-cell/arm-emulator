EXAMPLE_FIBONACCI = r"""_start:
    @ Calculate the 10th Fibonacci number
    MOV R0, #0      @ a = 0
    MOV R1, #1      @ b = 1
    MOV R2, #10     @ counter = 10

fib_loop:
    CMP R2, #0      @ Check if counter is 0
    BEQ end         @ If so, we are done
    
    ADD R3, R0, R1  @ next = a + b
    MOV R0, R1      @ a = b
    MOV R1, R3      @ b = next
    
    SUB R2, R2, #1  @ counter--
    B fib_loop

end:
    @ The answer is now in R0
    MOV R7, #1      @ Exit
    SVC 0

"""

EXAMPLE_BLINK = r"""_start:
    MOV R2, #0

loop:
    LDR R0, =led0
    BL turn_on

    LDR R0, =led0
    BL turn_off

    ADD R2, R2, #1
    CMP R2, #0x3
    BNE loop

    MOV R7, #1 @ Exit syscall
    MOV R0, #0 @ Exit code 0
    SVC 0      @ Supervisor call

turn_on:
    @ Save return address
    PUSH {LR}

    @ 1. Configure PA5 as Output
    @ We need bits 11:10 of MODER (Offset 0x00) to be '01'.
    @ Binary: ... 0000 0100 0000 0000
    @ Hex:    0x400
    MOV R1, #0x400
    STR R1, [R0]        @ Write to MODER (Offset 0)

    @ 2. Set PA5 High
    @ We need bit 5 of ODR (Offset 0x14) to be '1'.
    @ Binary: ... 0010 0000
    @ Hex:    0x20
    MOV R1, #0x20
    STR R1, [R0, #0x14] @ Write to ODR (Offset 20)

    @ Restore return address
    POP {PC}

turn_off:
    @ 1. Configure PA5 as Output
    MOV R1, #0x400
    STR R1, [R0]        @ Write to MODER (Offset 0)

    @ 2. Set PA5 Low
    MOV R1, #0x00
    STR R1, [R0, #0x14] @ Write to ODR (Offset 20)

    @ Or just use BX to return,
    @ since we don't need to restore LR, as we had no need to save it
    BX LR

"""
