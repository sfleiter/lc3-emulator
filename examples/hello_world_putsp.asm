;LC−3 Program that displays
;"HelloWorld!" to the console
.ORIG x3000
    LEA R0, HW                ;load address of string
;output string to console
    PUTSP
;end program
    HALT
HW .FILL x6548 ; eH
   .FILL x6c6c ; ll
   .FILL x206f ;  o
   .FILL x6f57 ; oW
   .FILL x6c72 ; lr
   .FILL x2164 ; !d
   .FILL x0000 ; empty
.END                          ;end assembler
