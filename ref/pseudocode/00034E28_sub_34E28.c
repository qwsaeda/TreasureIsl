/* ea:   0x00034E28 */
/* name: sub_34E28 */
/* size: 448 */
/* HEX-RAYS DECOMPILE FAILED -> disasm fallback */
PUSH            {R4,R5,LR}
MOV             R5, R0
CMP             R1, #4
BGT             locret_34FC0
LDR             R0, [R5,#0x108]
SUB             R2, R2, #1
CMP             R2, #8
ADD             R1, R0, R1,LSL#2
LDR             R0, [R5,#0x20]
LDR             R3, [R1,#-4]
LDR             R1, [R0,#0xA8]
STR             R1, [R5,#0x2DC]
BHI             locret_34FC0
MOV             R0, R2,LSL#1
ADD             R0, R0, PC
LDRH            R0, [R0,#4]
ADD             PC, PC, R0
EORSEQ          R0, R4, R0,LSL R0
POP             {R4,R5,PC}
STRHEQ          SP, [R6],-R12
ANDEQ           SP, R6, R8,LSR#21
DCD off_6DA94
ANDEQ           SP, R6, R0,LSL#21
ANDEQ           SP, R6, R12,ROR#20
ANDEQ           SP, R6, R8,ASR R10
ANDEQ           SP, R6, R4,ASR#20
ANDEQ           SP, R6, R0,LSR R10
ANDEQ           SP, R6, R12,LSL R10
