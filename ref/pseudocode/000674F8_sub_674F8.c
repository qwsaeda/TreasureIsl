/* ea:   0x000674F8 */
/* name: sub_674F8 */
/* size: 3908 */
/* HEX-RAYS DECOMPILE FAILED -> disasm fallback */
PUSH            {R4-R11,LR}
SUB             SP, SP, #0x48
MOV             R7, R0
STR             R1, [SP,#0x6C+var_4C]
STR             R7, [SP,#0x6C+var_44]
MOV             R10, R2
LDR             R5, [R7,#0x34]
LDR             R0, [R7,#0x30]
STR             R5, [SP,#0x6C+var_58]
MOV             R2, R5
CMP             R5, R0
LDR             R6, [R1]
SUBCC           R0, R0, R5
LDR             R11, [R1,#4]
SUBCC           R5, R0, #1
LDRCS           R0, [R7,#0x2C]
LDR             LR, [R7,#0x1C]
LDR             R4, [R7,#0x20]
SUBCS           R5, R0, R2
LDR             R1, [R7]
STR             R6, [SP,#0x6C+var_48]
STR             LR, [SP,#0x6C+var_54]
CMP             R1, #9
STR             R4, [SP,#0x6C+var_50]
BHI             loc_678F4
LDR             R9, =dword_80FE8
B               loc_67568
ANDEQ           R0, R8, R8,ROR#31
LDR             R8, =unk_7BD70
B               loc_67574
LDR             R2, [SP,#0x6C+var_58]
MOV             R3, #1
MOV             R0, R1,LSL#1
ADD             R0, R0, PC
LDRH            R0, [R0,#4]
ADD             PC, PC, R0
LDR             R1, [R7]
CMP             R1, #9
BLS             loc_67570
LDR             R3, [SP,#0x6C+var_4C]
STR             LR, [R7,#0x1C]
STR             R4, [R7,#0x20]
LDR             R0, [R3]
LDR             R1, [R3,#8]
STR             R6, [R3]
SUB             R1, R1, R0
LDR             R0, [SP,#0x6C+var_58]
ADD             R2, R1, R6
STR             R11, [R3,#4]
STR             R2, [R3,#8]
MOV             R1, R3
STR             R0, [R7,#0x34]
MOV             R2, #0xFFFFFFFE
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
MOV             R1, #0x3FC0
LDR             R5, [SP,#arg_20]
CMP             LR, #0xE
ORR             R1, R1, #0x3F
BCS             loc_67984
MOV             R3, #0
CMP             R11, #0
BEQ             loc_6812C
LDRB            R0, [R6],#1
MOV             R10, R3
SUB             R11, R11, #1
ORR             R4, R4, R0,LSL LR
STR             R6, [SP,#arg_24]
ADD             LR, LR, #8
STR             R4, [SP,#arg_1C]
CMP             LR, #0xE
STR             LR, [SP,#arg_18]
BCC             loc_67954
AND             R0, R4, R1
AND             R3, R0, #0x1F
STR             R0, [R7,#4]
CMP             R3, #0x1D
BHI             loc_68068
MOV             R0, R0,LSL#22
MOV             R1, R0,LSR#27
CMP             R1, #0x1D
BHI             loc_68068
ADD             R0, R1, R3
LDR             R3, [R5,#0x20]
ADD             R1, R0, #0x100
LDR             R0, [R5,#0x28]
MOV             R2, #4
ADD             R1, R1, #2
MOV             LR, PC
MOV             PC, R3
CMP             R0, #0
STR             R0, [R7,#0xC]
BEQ             loc_68020
LDR             LR, [SP,#arg_18]
MOV             R4, R4,LSR#14
MOV             R3, #0
STR             R4, [SP,#arg_1C]
SUB             LR, LR, #0xE
STR             R3, [R7,#8]
MOV             R0, #4
STR             LR, [SP,#arg_18]
STR             R0, [R7]
B               loc_67A04
LDR             R5, [SP,#arg_20]
MOV             R3, #0
LDR             R0, [R7,#4]
LDR             R2, [R7,#8]
MOV             R0, R0,LSR#10
ADD             R1, R0, #4
CMP             R2, R1
BCS             loc_67A9C
CMP             LR, #3
BCS             loc_67A54
CMP             R11, #0
BEQ             loc_6812C
LDRB            R0, [R6],#1
MOV             R10, R3
SUB             R11, R11, #1
ORR             R4, R4, R0,LSL LR
STR             R6, [SP,#arg_24]
ADD             LR, LR, #8
STR             R4, [SP,#arg_1C]
CMP             LR, #3
STR             LR, [SP,#arg_18]
BCC             loc_67A24
LDR             R0, [R7,#8]
AND             R2, R4, #7
LDR             R1, [R7,#0xC]
MOV             R4, R4,LSR#3
LDR             R0, [R8,R0,LSL#2]
SUB             LR, LR, #3
STR             R4, [SP,#arg_1C]
STR             R2, [R1,R0,LSL#2]
LDR             R1, [R7,#8]
STR             LR, [SP,#arg_18]
ADD             R0, R1, #1
STR             R0, [R7,#8]
LDR             R0, [R7,#4]
LDR             R2, [R7,#8]
MOV             R0, R0,LSR#10
ADD             R1, R0, #4
CMP             R2, R1
BCC             loc_67A1C
LDR             R0, [R7,#8]
CMP             R0, #0x13
BCS             loc_67AD0
LDR             R0, [R7,#8]
LDR             R1, [R7,#0xC]
LDR             R0, [R8,R0,LSL#2]
STR             R3, [R1,R0,LSL#2]
LDR             R1, [R7,#8]
ADD             R0, R1, #1
MOV             R1, R0
STR             R0, [R7,#8]
CMP             R1, #0x13
BCC             loc_67AA8
MOV             R0, #7
LDR             R3, [R7,#0x24]
STR             R0, [R7,#0x10]
ADD             R2, R7, #0x14
LDR             R0, [R7,#0xC]
ADD             R1, R7, #0x10
STR             R5, [SP,#arg_0]
BL              sub_68FF8
MOVS            R4, R0
BNE             loc_680BC
MOV             R0, #0
LDR             LR, [SP,#arg_18]
STR             R0, [R7,#8]
MOV             R0, #5
LDR             R4, [SP,#arg_1C]
STR             R0, [R7]
B               loc_67B18
LDR             R5, [SP,#arg_20]
LDR             R0, [R7,#4]
LDR             R3, [R7,#8]
MOV             R1, R0,LSL#22
AND             R0, R0, #0x1F
ADD             R1, R0, R1,LSR#27
ADD             R2, R1, #0x100
ADD             R0, R2, #2
CMP             R3, R0
BCS             loc_67D14
B               loc_67B48
LDR             LR, [SP,#arg_18]
LDR             R4, [SP,#arg_1C]
LDR             R1, [R7,#0x10]
CMP             LR, R1
BCS             loc_67B88
MOV             R3, #0
CMP             R11, #0
BEQ             loc_6812C
LDRB            R0, [R6],#1
MOV             R10, R3
SUB             R11, R11, #1
ORR             R4, R4, R0,LSL LR
STR             R6, [SP,#arg_24]
ADD             LR, LR, #8
STR             R4, [SP,#arg_1C]
CMP             LR, R1
STR             LR, [SP,#arg_18]
BCC             loc_67B58
LDR             R0, [R9,R1,LSL#2]
LDR             R1, [R7,#0x14]
AND             R2, R0, R4
ADD             R0, R1, R2,LSL#3
LDR             R7, [R0,#4]
LDRB            R6, [R0,#1]
CMP             R7, #0x10
BCS             loc_67BE0
LDR             R3, [SP,#arg_28]
MOV             R4, R4,LSR R6
SUB             LR, LR, R6
LDR             R6, [SP,#arg_24]
LDR             R1, [R3,#8]
LDR             R0, [R3,#0xC]
STR             R4, [SP,#arg_1C]
STR             R7, [R0,R1,LSL#2]
MOV             R7, R3
LDR             R1, [R7,#8]
STR             LR, [SP,#arg_18]
ADD             R0, R1, #1
STR             R0, [R7,#8]
B               loc_67CF0
CMP             R7, #0x12
BNE             loc_67BF8
MOV             R2, #7
MOV             R4, R2
MOV             R3, #0xB
B               loc_67C04
MOV             R0, #3
SUB             R4, R7, #0xE
MOV             R3, R0
ADD             R1, R4, R6
LDR             R2, [SP,#arg_1C]
CMP             LR, R1
BCS             loc_67C4C
LDR             R5, [SP,#arg_24]
CMP             R11, #0
BEQ             loc_68170
MOV             R0, #0
MOV             R10, R0
LDRB            R0, [R5],#1
SUB             R11, R11, #1
ORR             R2, R2, R0,LSL LR
STR             R5, [SP,#arg_24]
ADD             LR, LR, #8
STR             R2, [SP,#arg_1C]
CMP             LR, R1
STR             LR, [SP,#arg_18]
BCC             loc_67C18
MOV             R2, R2,LSR R6
LDR             R0, [R9,R4,LSL#2]
AND             R1, R0, R2
MOV             R0, R2,LSR R4
ADD             R5, R3, R1
STR             R0, [SP,#arg_1C]
SUB             R0, LR, R4
SUB             R0, R0, R6
LDR             R6, [SP,#arg_28]
STR             R0, [SP,#arg_18]
LDR             R2, [R6,#4]
LDR             R4, [R6,#8]
AND             R0, R2, #0x1F
MOV             R1, R2,LSL#22
ADD             R1, R0, R1,LSR#27
ADD             R2, R1, #0x100
ADD             R3, R2, #2
ADD             R0, R5, R4
CMP             R0, R3
BHI             loc_681C0
CMP             R7, #0x10
BNE             loc_67CC4
CMP             R4, #1
BCC             loc_681C0
MOV             R7, R6
LDR             R0, [R7,#0xC]
MOV             R3, #0
ADD             R1, R0, R4,LSL#2
LDR             R2, [R1,#-4]
B               loc_67CD0
MOV             R3, #0
LDR             R7, [SP,#arg_28]
MOV             R2, R3
LDR             R6, [SP,#arg_24]
LDR             R0, [R7,#0xC]
SUBS            R5, R5, #1
STR             R2, [R0,R4,LSL#2]
ADD             R4, R4, #1
BNE             loc_67CD4
LDR             R5, [SP,#arg_20]
STR             R4, [R7,#8]
LDR             R0, [R7,#4]
LDR             R3, [R7,#8]
MOV             R1, R0,LSL#22
AND             R0, R0, #0x1F
ADD             R1, R0, R1,LSR#27
ADD             R2, R1, #0x100
ADD             R0, R2, #2
CMP             R3, R0
BCC             loc_67B40
MOV             R0, #0
LDR             R4, [R7,#4]
STR             R0, [R7,#0x14]
MOV             R0, #9
STR             R0, [SP,#arg_2C]
MOV             R0, #6
STR             R0, [SP,#arg_24]
ADD             R1, SP, #arg_40
LDR             R0, [R7,#0x24]
ADD             R3, SP, #arg_2C
STR             R1, [SP,#arg_8]
ADD             R1, SP, #arg_24
STR             R0, [SP,#arg_C]
ADD             R0, SP, #arg_44
STR             R0, [SP,#arg_4]
MOV             R0, R4,LSL#22
STR             R1, [SP,#arg_0]
MOV             R1, R0,LSR#27
AND             R0, R4, #0x1F
LDR             R2, [R7,#0xC]
ADD             R0, R0, #0x100
STR             R5, [SP,#arg_10]
ADD             R1, R1, #1
ADD             R0, R0, #1
BL              sub_69708
MOVS            R4, R0
BNE             loc_68238
LDR             R3, [SP,#arg_40]
LDR             R2, [SP,#arg_44]
LDR             R1, [SP,#arg_24]
LDR             R0, [SP,#arg_2C]
STR             R5, [SP,#arg_0]
BL              sub_68628
CMP             R0, #0
BEQ             loc_682A8
STR             R0, [R7,#4]
LDR             R1, [R7,#0xC]
LDR             R2, [R5,#0x24]
LDR             R0, [R5,#0x28]
MOV             LR, PC
MOV             PC, R2
MOV             R3, #6
LDR             R4, [SP,#arg_1C]
LDR             LR, [SP,#arg_18]
STR             R3, [R7]
B               loc_67DD0
LDR             R5, [SP,#arg_20]
STR             LR, [R7,#0x1C]
STR             R4, [R7,#0x20]
LDR             R0, [R5]
LDR             R1, [R5,#8]
STR             R6, [R5]
SUB             R1, R1, R0
LDR             R0, [SP,#arg_14]
ADD             R2, R1, R6
STR             R11, [R5,#4]
STR             R2, [R5,#8]
MOV             R1, R5
STR             R0, [R7,#0x34]
MOV             R2, R10
MOV             R0, R7
BL              sub_68688
CMP             R0, #1
MOV             R1, R5
BNE             loc_682F4
MOV             R0, #0
MOV             R10, R0
LDR             R0, [R7,#4]
BL              sub_68FD4
LDR             R1, [R7,#0x34]
LDR             R0, [R7,#0x30]
LDR             R6, [R5]
CMP             R1, R0
LDR             R11, [R5,#4]
SUBCC           R0, R0, R1
LDR             LR, [R7,#0x1C]
SUBCC           R5, R0, #1
LDRCS           R0, [R7,#0x2C]
LDR             R4, [R7,#0x20]
SUBCS           R5, R0, R1
LDR             R0, [R7,#0x18]
STR             R6, [SP,#arg_24]
STR             LR, [SP,#arg_18]
CMP             R0, #0
STR             R4, [SP,#arg_1C]
STR             R1, [SP,#arg_14]
BNE             loc_68308
MOV             R1, #0
STR             R1, [R7]
B               loc_678E8
STR             R4, [R7,#0x20]
LDR             R0, [SP,#arg_18]
LDR             R4, [SP,#arg_20]
STR             R0, [R7,#0x1C]
LDR             R0, [R4]
LDR             R1, [R4,#8]
STR             R6, [R4]
SUB             R1, R1, R0
LDR             R0, [SP,#arg_14]
ADD             R2, R1, R6
STR             R11, [R4,#4]
STR             R2, [R4,#8]
MOV             R1, R4
STR             R0, [R7,#0x34]
MOV             R2, #0xFFFFFFFC
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
MOV             R0, #9
LDR             R8, [SP,#-0x6C+arg_8C]
STR             R0, [R7]
MOV             R1, R4,LSR#3
LDR             R0, =aInvalidBlockTy; "invalid block type"
STR             R0, [R8,#0x18]
SUB             R0, LR, #3
STR             R0, [R7,#0x1C]
STR             R1, [R7,#0x20]
LDR             R0, [R8]
LDR             R1, [R8,#8]
STR             R6, [R8]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x6C+arg_80]
ADD             R2, R1, R6
STR             R11, [R8,#4]
STR             R2, [R8,#8]
MOV             R1, R8
STR             R0, [R7,#0x34]
MOV             R2, #0xFFFFFFFD
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
STR             R4, [R7,#0x20]
LDR             R4, [SP,#-0xD8+arg_F8]
STR             LR, [R7,#0x1C]
LDR             R0, [R4]
LDR             R1, [R4,#8]
STR             R6, [R4]
SUB             R1, R1, R0
LDR             R0, [SP,#-0xD8+arg_EC]
ADD             R2, R1, R6
MOV             R1, #0
STR             R2, [R4,#8]
STR             R1, [R4,#4]
MOV             R2, R10
STR             R0, [R7,#0x34]
MOV             R1, R4
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
MOV             R0, #9
LDR             R8, [SP,#-0x144+arg_164]
STR             R0, [R7]
LDR             R0, =aInvalidStoredB; "invalid stored block lengths"
STR             R0, [R8,#0x18]
STR             LR, [R7,#0x1C]
STR             R4, [R7,#0x20]
LDR             R0, [R8]
LDR             R1, [R8,#8]
STR             R6, [R8]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x144+arg_158]
ADD             R2, R1, R6
STR             R11, [R8,#4]
STR             R2, [R8,#8]
MOV             R1, R8
STR             R0, [R7,#0x34]
MOV             R2, #0xFFFFFFFD
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
LDR             R0, [SP,#-0x1B0+arg_1C8]
LDR             R1, [SP,#-0x1B0+arg_1CC]
LDR             R4, [SP,#-0x1B0+arg_1D0]
STR             R1, [R7,#0x20]
STR             R0, [R7,#0x1C]
LDR             R0, [R4]
LDR             R1, [R4,#8]
STR             R6, [R4]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x1B0+arg_1C4]
ADD             R2, R1, R6
MOV             R1, #0
STR             R2, [R4,#8]
STR             R1, [R4,#4]
MOV             R2, R10
STR             R0, [R7,#0x34]
MOV             R1, R4
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
LDR             R0, [SP,#-0x21C+arg_234]
STR             R4, [R7,#0x20]
STR             R0, [R7,#0x1C]
LDR             R0, [R5]
LDR             R1, [R5,#8]
STR             R11, [R5,#4]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x21C+arg_230]
ADD             R2, R1, R6
STR             R6, [R5]
STR             R2, [R5,#8]
MOV             R1, R5
STR             R0, [R7,#0x34]
MOV             R2, #0xFFFFFFFC
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
MOV             R0, #9
STR             R0, [R7]
LDR             R0, =aTooManyLengthO; "too many length or distance symbols"
STR             R0, [R5,#0x18]
STR             LR, [R7,#0x1C]
STR             R4, [R7,#0x20]
LDR             R0, [R5]
LDR             R1, [R5,#8]
STR             R11, [R5,#4]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x288+arg_29C]
ADD             R2, R1, R6
STR             R6, [R5]
STR             R2, [R5,#8]
MOV             R1, R5
STR             R0, [R7,#0x34]
MOV             R2, #0xFFFFFFFD
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
CMN             R4, #3
BNE             loc_6825C
LDR             R1, [R7,#0xC]
LDR             R2, [R5,#0x24]
LDR             R0, [R5,#0x28]
MOV             LR, PC
MOV             PC, R2
MOV             R0, #9
LDR             R1, [SP,#-0x2F4+arg_310]
STR             R0, [R7]
LDR             R0, [SP,#-0x2F4+arg_30C]
STR             R1, [R7,#0x20]
STR             R0, [R7,#0x1C]
LDR             R0, [R5]
LDR             R1, [R5,#8]
STR             R6, [R5]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x2F4+arg_308]
ADD             R2, R1, R6
STR             R11, [R5,#4]
STR             R2, [R5,#8]
MOV             R1, R5
STR             R0, [R7,#0x34]
MOV             R2, R4
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
STR             LR, [R7,#0x1C]
STR             R4, [R7,#0x20]
LDR             R0, [R5]
LDR             R1, [R5,#8]
STR             R3, [R5,#4]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x360+arg_374]
ADD             R2, R1, R6
STR             R6, [R5]
STR             R2, [R5,#8]
MOV             R1, R5
STR             R0, [R7,#0x34]
MOV             R2, R10
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
LDR             R3, [SP,#-0x3CC+arg_3F4]
LDR             R4, [SP,#-0x3CC+arg_3EC]
STR             R2, [R3,#0x20]
STR             LR, [R3,#0x1C]
LDR             R0, [R4]
LDR             R1, [R4,#8]
STR             R5, [R4]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x3CC+arg_3E0]
ADD             R2, R1, R5
MOV             R1, #0
STR             R2, [R4,#8]
STR             R1, [R4,#4]
MOV             R2, R10
STR             R0, [R3,#0x34]
MOV             R1, R4
MOV             R0, R3
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
LDR             R8, [SP,#-0x438+arg_458]
LDR             R1, [R6,#0xC]
LDR             R2, [R8,#0x24]
LDR             R0, [R8,#0x28]
MOV             LR, PC
MOV             PC, R2
MOV             R0, #9
LDR             R1, [SP,#-0x438+arg_454]
STR             R0, [R6]
LDR             R0, =aInvalidBitLeng; "invalid bit length repeat"
LDR             R3, [SP,#-0x438+arg_45C]
STR             R0, [R8,#0x18]
LDR             R0, [SP,#-0x438+arg_450]
STR             R1, [R6,#0x20]
STR             R0, [R6,#0x1C]
LDR             R0, [R8]
LDR             R1, [R8,#8]
STR             R3, [R8]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x438+arg_44C]
ADD             R2, R1, R3
STR             R11, [R8,#4]
STR             R2, [R8,#8]
MOV             R1, R8
STR             R0, [R6,#0x34]
MOV             R2, #0xFFFFFFFD
MOV             R0, R6
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
CMN             R4, #3
BNE             loc_6825C
LDR             R1, [R7,#0xC]
LDR             R2, [R5,#0x24]
LDR             R0, [R5,#0x28]
MOV             LR, PC
MOV             PC, R2
MOV             R0, #9
STR             R0, [R7]
LDR             R0, [SP,#-0x4A4+arg_4BC]
LDR             R1, [SP,#-0x4A4+arg_4C0]
STR             R0, [R7,#0x1C]
STR             R1, [R7,#0x20]
LDR             R0, [R5]
LDR             R1, [R5,#8]
STR             R6, [R5]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x4A4+arg_4B8]
ADD             R2, R1, R6
STR             R11, [R5,#4]
STR             R2, [R5,#8]
MOV             R1, R5
STR             R0, [R7,#0x34]
MOV             R2, R4
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
LDR             R0, [SP,#-0x510+arg_528]
LDR             R1, [SP,#-0x510+arg_52C]
STR             R0, [R7,#0x1C]
STR             R1, [R7,#0x20]
LDR             R0, [R5]
LDR             R1, [R5,#8]
STR             R6, [R5]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x510+arg_524]
ADD             R2, R1, R6
STR             R11, [R5,#4]
STR             R2, [R5,#8]
MOV             R1, R5
STR             R0, [R7,#0x34]
MOV             R2, #0xFFFFFFFC
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
MOV             R2, R0
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
MOV             R2, #7
STR             R2, [R7]
B               loc_68318
LDR             R1, [SP,#-0x5E8+arg_5FC]
LDR             R8, [SP,#-0x5E8+arg_608]
MOV             R2, R10
STR             R1, [R7,#0x34]
MOV             R0, R7
MOV             R1, R8
BL              sub_69990
MOV             R3, R0
LDR             LR, [R7,#0x34]
LDR             R0, [R7,#0x30]
STR             LR, [SP,#-0x5E8+arg_5FC]
CMP             R0, LR
BEQ             loc_6838C
LDR             R0, [SP,#-0x5E8+arg_600]
STR             R4, [R7,#0x20]
STR             R0, [R7,#0x1C]
LDR             R0, [R8]
LDR             R1, [R8,#8]
STR             R6, [R8]
SUB             R1, R1, R0
STR             R11, [R8,#4]
ADD             R2, R1, R6
STR             R2, [R8,#8]
MOV             R1, R8
MOV             R2, R3
STR             LR, [R7,#0x34]
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
MOV             R0, #8
LDR             LR, [SP,#-0x654+arg_66C]
STR             R0, [R7]
LDR             R5, [SP,#-0x654+arg_674]
STR             LR, [R7,#0x1C]
STR             R4, [R7,#0x20]
LDR             R0, [R5]
LDR             R1, [R5,#8]
STR             R6, [R5]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x654+arg_668]
ADD             R2, R1, R6
STR             R11, [R5,#4]
STR             R2, [R5,#8]
MOV             R1, R5
STR             R0, [R7,#0x34]
MOV             R2, #1
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
LDR             R3, [SP,#-0x6C0+arg_6E0]
STR             LR, [R7,#0x1C]
STR             R4, [R7,#0x20]
LDR             R0, [R3]
LDR             R1, [R3,#8]
STR             R6, [R3]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x6C0+arg_6D4]
ADD             R2, R1, R6
STR             R11, [R3,#4]
STR             R2, [R3,#8]
MOV             R1, R3
STR             R0, [R7,#0x34]
MOV             R2, #0xFFFFFFFD
MOV             R0, R7
BL              sub_69990
ADD             SP, SP, #0x48 ; 'H'
POP             {R4-R11,PC}
ANDEQ           PC, R7, R12,LSR#25
ANDEQ           PC, R7, R8,ASR#25
ANDEQ           PC, R7, R12,ROR#25
ANDEQ           PC, R7, R12,LSL#26
ANDEQ           R11, R7, R0,ROR SP
