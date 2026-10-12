/* ea:   0x00066DBC */
/* name: sub_66DBC */
/* size: 1460 */
/* HEX-RAYS DECOMPILE FAILED -> disasm fallback */
PUSH            {R4-R11,LR}
SUB             SP, SP, #4
MOV             R5, R0
STR             R5, [SP,#0x28+var_28]
CMP             R5, #0
BEQ             loc_67174
LDR             R0, [R5,#0x1C]
MOVS            R2, R0
BEQ             loc_67174
LDR             R0, [R5]
CMP             R0, #0
BEQ             loc_67174
CMP             R1, #4
LDR             R1, [R2]
MOV             R0, #0
MOV             R9, #0xFFFFFFFB
MOVNE           R9, R0
MOV             R11, #0xFFFFFFFB
CMP             R1, #0xD
BHI             loc_67174
LDR             R10, =aIncorrectDataC; "incorrect data check"
MOV             R4, #5
LDR             R8, =aIncorrectHeade; "incorrect header check"
LDR             R7, =aInvalidWindowS; "invalid window size"
LDR             R6, =aUnknownCompres; "unknown compression method"
MOV             LR, #2
MOV             R0, R1,LSL#1
ADD             R0, R0, PC
LDRH            R0, [R0,#4]
ADD             PC, PC, R0
ADCEQ           R0, R12, R8,LSL R0
MOV             LR, #0xFFFFFFFE
MOV             R0, LR
ADD             SP, SP, #4
POP             {R4-R11,PC}
MOV             LR, R11
MOV             R0, LR
ADD             SP, SP, #4
POP             {R4-R11,PC}
LDR             R5, [SP,#-0x28+arg_28]
LDR             R0, [R5,#0x1C]
STR             LR, [R0]
LDR             R0, [R5,#4]
CMP             R0, #0
BEQ             loc_67184
SUB             R2, R0, #1
LDR             R0, [R5,#8]
STR             R2, [R5,#4]
MOV             R11, R9
ADD             R1, R0, #1
LDR             R0, [R5]
STR             R1, [R5,#8]
LDRB            R1, [R0]
LDR             R0, [R5,#0x1C]
MOV             R2, R1,LSL#24
STR             R2, [R0,#8]
LDR             R1, [R5]
ADD             R0, R1, #1
LDR             R1, [R5,#0x1C]
STR             R0, [R5]
MOV             R0, #3
STR             R0, [R1]
LDR             R0, [R5,#4]
CMP             R0, #0
BEQ             loc_67184
SUB             R2, R0, #1
LDR             R0, [R5,#8]
LDR             R3, [R5,#0x1C]
MOV             R11, R9
ADD             R1, R0, #1
LDR             R0, [R5]
STR             R1, [R5,#8]
STR             R2, [R5,#4]
LDRB            R2, [R0]
LDR             R1, [R3,#8]
ADD             R0, R1, R2,LSL#16
STR             R0, [R3,#8]
LDR             R1, [R5]
ADD             R0, R1, #1
LDR             R1, [R5,#0x1C]
STR             R0, [R5]
MOV             R0, #4
STR             R0, [R1]
LDR             R0, [R5,#4]
CMP             R0, #0
BEQ             loc_67184
SUB             R2, R0, #1
LDR             R0, [R5,#8]
LDR             R3, [R5,#0x1C]
MOV             R11, R9
ADD             R1, R0, #1
LDR             R0, [R5]
STR             R1, [R5,#8]
STR             R2, [R5,#4]
LDRB            R2, [R0]
LDR             R1, [R3,#8]
ADD             R0, R1, R2,LSL#8
STR             R0, [R3,#8]
LDR             R1, [R5]
ADD             R0, R1, #1
LDR             R1, [R5,#0x1C]
STR             R0, [R5]
MOV             R0, #5
STR             R0, [R1]
LDR             R0, [R5,#4]
CMP             R0, #0
BEQ             loc_67184
SUB             R2, R0, #1
LDR             R0, [R5,#8]
LDR             R3, [R5,#0x1C]
ADD             R1, R0, #1
LDR             R0, [R5]
STR             R1, [R5,#8]
STR             R2, [R5,#4]
LDRB            R2, [R0]
LDR             R1, [R3,#8]
ADD             R0, R2, R1
STR             R0, [R3,#8]
MOV             R2, #6
LDR             R1, [R5]
ADD             R0, R1, #1
LDR             R1, [R5,#0x1C]
STR             R0, [R5]
LDR             R0, [R1,#8]
STR             R0, [R5,#0x30]
MOV             R0, LR
STR             R2, [R1]
ADD             SP, SP, #4
POP             {R4-R11,PC}
LDR             R0, [R5,#0x1C]
MOV             R1, #0xD
MOV             R2, #0
STR             R1, [R0]
MOV             LR, #0xFFFFFFFE
LDR             R1, =aNeedDictionary; "need dictionary"
LDR             R0, [R5,#0x1C]
STR             R1, [R5,#0x18]
STR             R2, [R0,#4]
MOV             R0, LR
ADD             SP, SP, #4
POP             {R4-R11,PC}
LDR             R0, [R5,#0x1C]
MOV             R1, #0xC
STR             R1, [R0]
MOV             R0, #1
MOV             LR, R0
ADD             SP, SP, #4
POP             {R4-R11,PC}
MOV             LR, #0xFFFFFFFD
MOV             R0, LR
ADD             SP, SP, #4
POP             {R4-R11,PC}
ANDEQ           PC, R7, R12,LSR R12
STMDAEQ         R2, {R0,R2,R7,R12}^
ANDEQ           PC, R7, R12,ASR#24
ANDEQ           PC, R7, R8,ROR#24
ANDEQ           PC, R7, R12,ROR R12
DCD aIncorrectDataC
