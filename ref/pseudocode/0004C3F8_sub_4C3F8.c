/* ea:   0x0004C3F8 */
/* name: sub_4C3F8 */
/* size: 300 */
int __fastcall sub_4C3F8(int a1, int a2, int a3)
{
  unsigned int v7; // r3
  _WORD *v8; // r4
  unsigned int v9; // r2
  unsigned __int8 v10; // r6
  int v11; // r0
  __int16 v12; // r0

  if ( a3 == 0 )
    return 0;
  v7 = (unsigned __int16)(((unsigned int)sub_16C9C((unsigned __int16 *)(a1 + 136), 1) >> 1) - *(_WORD *)(a2 + 34));
  v8 = (_WORD *)(a1 + 140);
  v9 = *(unsigned __int16 *)(a1 + 140);
  v10 = 0;
  if ( v7 < v9 )
    *v8 = v9 - v7;
  else
    *v8 = 0;
  v11 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  sub_16D78((unsigned __int16 *)(a1 + 136), ((unsigned int)(v11 + 1) >> 1) - *(_WORD *)(a2 + 32));
  v12 = 0;
  if ( (unsigned __int16)(*(_WORD *)(a3 + 24) - *(_WORD *)(a2 + 40)) <= (unsigned int)*(unsigned __int16 *)(a3 + 24) )
    v12 = *(_WORD *)(a2 + 40);
  *(_WORD *)(a3 + 24) = v12;
  sub_31200(a3, *(_WORD *)(a3 + 22) - *(_WORD *)(a2 + 38));
  if ( *(_DWORD *)(a2 + 28) == *(_DWORD *)(a1 + 8) )
    return 1;
  return v10;
}

