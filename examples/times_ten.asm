.ORIG x3000
;----------------------------------------
; Example: Times 10 (looping)
;----------------------------------------
LD R0, FACTOR ; load the FACTOR we want to multiply with 10
LD R3, ZERO ; stort with zero in R3
LD R2, LOOP_COUNT ; load the loop max

LOOP_START
    ADD  R3, R3, R0; add R0 for each loop
    ADD  R2, R2, #-1; decrement R2 each loop
    BRnp LOOP_START ; loop back if last operation was not zero (R2)
    ; else we are done

HALT; R3 should have our result (R0, R1 will be changed by the trap)

FACTOR
    .FILL #3; the base value to be multiplied by 10
LOOP_COUNT
    .FILL #10; the FACTOR 10
ZERO
    .FILL #0; the FACTOR zero
.END
