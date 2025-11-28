;LC−3 Program that displays
;"HelloWorld!" to the console
.ORIG x3000
    LEA R0, HW                ;load address of string
;output string to console
    PUTS
;end program
    HALT
HW  .STRINGZ "HelloWorld!"
.END                          ;end assembler
