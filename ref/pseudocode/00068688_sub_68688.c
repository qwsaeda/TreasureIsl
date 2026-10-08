/* ea:   0x00068688 */
/* name: sub_68688 */
/* size: 2380 */
/* HEX-RAYS DECOMPILE FAILED -> disasm fallback */
PUSH            {R4-R11,LR}
SUB             SP, SP, #0x18
MOV             R10, R0
MOV             R11, R1
MOV             R7, R2
LDR             R9, [R10,#0x34]
LDR             R0, [R10,#0x30]
LDR             R8, [R10,#4]
CMP             R9, R0
LDR             R4, [R11]
SUBCC           R0, R0, R9
LDR             LR, [R11,#4]
SUBCC           R6, R0, #1
LDRCS           R0, [R10,#0x2C]
LDR             R2, [R10,#0x1C]
LDR             R3, [R10,#0x20]
SUBCS           R6, R0, R9
LDR             R1, [R8]
STR             R4, [SP,#0x3C+var_28]
STR             LR, [SP,#0x3C+var_2C]
CMP             R1, #9
STR             R2, [SP,#0x3C+var_30]
STR             R3, [SP,#0x3C+var_34]
BHI             loc_68C9C
LDR             R5, =dword_80FE8
MOV             R0, R1,LSL#1
ADD             R0, R0, PC
LDRH            R0, [R0,#4]
ADD             PC, PC, R0
RSCEQ           R0, R12, R0,LSL R0
STR             R2, [R10,#0x1C]
STR             R3, [R10,#0x20]
LDR             R0, [R11]
LDR             R1, [R11,#8]
STR             LR, [R11,#4]
SUB             R1, R1, R0
STR             R4, [R11]
ADD             R2, R1, R4
STR             R2, [R11,#8]
MOV             R0, R10
MOV             R2, #0xFFFFFFFE
STR             R9, [R10,#0x34]
MOV             R1, R11
BL              sub_69990
ADD             SP, SP, #0x18
POP             {R4-R11,PC}
LDR             R0, =aInvalidLiteral; "invalid literal/length code"
MOV             R1, #9
STR             R1, [R8]
STR             R0, [R11,#0x18]
LDR             R0, [SP,#arg_8]
STR             R2, [R10,#0x1C]
STR             R0, [R10,#0x20]
LDR             R0, [R11]
LDR             R1, [R11,#8]
STR             LR, [R11,#4]
SUB             R1, R1, R0
STR             R4, [R11]
ADD             R2, R1, R4
STR             R2, [R11,#8]
MOV             R0, R10
MOV             R2, #0xFFFFFFFD
STR             R9, [R10,#0x34]
MOV             R1, R11
BL              sub_69990
ADD             SP, SP, #0x18
POP             {R4-R11,PC}
STR             R2, [R10,#0x1C]
STR             R3, [R10,#0x20]
LDR             R0, [R11]
LDR             R1, [R11,#8]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x3C+arg_50]
ADD             R2, R1, R0
STR             R0, [R11]
MOV             R0, #0
STR             R2, [R11,#8]
STR             R0, [R11,#4]
MOV             R2, R7
MOV             R0, R10
STR             R9, [R10,#0x34]
MOV             R1, R11
BL              sub_69990
ADD             SP, SP, #0x18
POP             {R4-R11,PC}
STR             R2, [R10,#0x1C]
STR             R3, [R10,#0x20]
LDR             R0, [R11]
LDR             R1, [R11,#8]
STR             R4, [R11]
SUB             R1, R1, R0
ADD             R2, R1, R4
MOV             R0, #0
STR             R2, [R11,#8]
STR             R0, [R11,#4]
MOV             R2, R7
MOV             R0, R10
STR             R9, [R10,#0x34]
MOV             R1, R11
BL              sub_69990
ADD             SP, SP, #0x18
POP             {R4-R11,PC}
LDR             R0, =aInvalidDistanc; "invalid distance code"
MOV             R1, #9
STR             R1, [R8]
STR             R0, [R11,#0x18]
LDR             R0, [SP,#-0xB4+arg_BC]
STR             R2, [R10,#0x1C]
STR             R0, [R10,#0x20]
LDR             R0, [R11]
LDR             R1, [R11,#8]
STR             LR, [R11,#4]
SUB             R1, R1, R0
STR             R4, [R11]
ADD             R2, R1, R4
STR             R2, [R11,#8]
MOV             R0, R10
MOV             R2, #0xFFFFFFFD
STR             R9, [R10,#0x34]
MOV             R1, R11
BL              sub_69990
ADD             SP, SP, #0x18
POP             {R4-R11,PC}
STR             R2, [R10,#0x1C]
STR             R1, [R10,#0x20]
LDR             R0, [R11]
LDR             R1, [R11,#8]
STR             R4, [R11]
SUB             R1, R1, R0
ADD             R2, R1, R4
MOV             R0, #0
STR             R2, [R11,#8]
STR             R0, [R11,#4]
MOV             R2, R7
MOV             R0, R10
STR             R9, [R10,#0x34]
MOV             R1, R11
BL              sub_69990
ADD             SP, SP, #0x18
POP             {R4-R11,PC}
LDR             R0, [SP,#-0x12C+arg_138]
STR             R0, [R10,#0x1C]
LDR             R0, [SP,#-0x12C+arg_134]
STR             R0, [R10,#0x20]
LDR             R0, [R11]
LDR             R1, [R11,#8]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x12C+arg_140]
ADD             R2, R1, R0
STR             R0, [R11]
LDR             R0, [SP,#-0x12C+arg_13C]
MOV             R1, R11
STR             R2, [R11,#8]
MOV             R2, R3
STR             R0, [R11,#4]
MOV             R0, R10
STR             R9, [R10,#0x34]
BL              sub_69990
ADD             SP, SP, #0x18
POP             {R4-R11,PC}
CMP             R2, #7
BLS             loc_68EC0
SUB             R2, R2, #8
ADD             LR, LR, #1
STR             R2, [SP,#-0x168+arg_174]
STR             LR, [SP,#-0x168+arg_178]
SUB             R4, R4, #1
MOV             R2, R7
STR             R9, [R10,#0x34]
MOV             R1, R11
MOV             R0, R10
BL              sub_69990
LDR             R9, [R10,#0x34]
MOV             R3, R0
LDR             R0, [R10,#0x30]
CMP             R0, R9
BEQ             loc_68F34
LDR             R0, [SP,#-0x168+arg_174]
STR             R0, [R10,#0x1C]
LDR             R0, [SP,#-0x168+arg_170]
STR             R0, [R10,#0x20]
LDR             R0, [R11]
LDR             R1, [R11,#8]
STR             R4, [R11]
SUB             R1, R1, R0
LDR             R0, [SP,#-0x168+arg_178]
ADD             R2, R1, R4
STR             R0, [R11,#4]
MOV             R1, R11
STR             R2, [R11,#8]
MOV             R0, R10
MOV             R2, R3
STR             R9, [R10,#0x34]
BL              sub_69990
ADD             SP, SP, #0x18
POP             {R4-R11,PC}
MOV             R0, #8
LDR             LR, [SP,#-0x1A4+arg_1B4]
LDR             R2, [SP,#-0x1A4+arg_1B0]
LDR             R3, [SP,#-0x1A4+arg_1AC]
STR             R0, [R8]
STR             R2, [R10,#0x1C]
STR             R3, [R10,#0x20]
LDR             R0, [R11]
LDR             R1, [R11,#8]
STR             LR, [R11,#4]
SUB             R1, R1, R0
STR             R4, [R11]
ADD             R2, R1, R4
STR             R2, [R11,#8]
MOV             R0, R10
MOV             R2, #1
STR             R9, [R10,#0x34]
MOV             R1, R11
BL              sub_69990
ADD             SP, SP, #0x18
POP             {R4-R11,PC}
STR             R2, [R10,#0x1C]
STR             R3, [R10,#0x20]
LDR             R0, [R11]
LDR             R1, [R11,#8]
STR             LR, [R11,#4]
SUB             R1, R1, R0
STR             R4, [R11]
ADD             R2, R1, R4
STR             R2, [R11,#8]
MOV             R0, R10
MOV             R2, #0xFFFFFFFD
STR             R9, [R10,#0x34]
MOV             R1, R11
BL              sub_69990
ADD             SP, SP, #0x18
POP             {R4-R11,PC}
STRHEQ          PC, [R7],-R8
DCD aInvalidLiteral
ANDEQ           R0, R8, R8,ROR#31
