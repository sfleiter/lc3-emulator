;LC−3 Program that reads a char and presents the output on screen
.ORIG x3000
;read char in R0
    GETC
;prints char from R0
    OUT
;end program
    HALT
.END
