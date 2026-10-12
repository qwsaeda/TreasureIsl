/* ea:   0x0004C524 */
/* name: sub_4C524 */
/* size: 332 */
int __fastcall sub_4C524(int a1, int a2, int a3)
{
  unsigned int v7; // r3
  _WORD *v8; // r4
  unsigned int v9; // r2
  unsigned __int8 v10; // r7
  int v11; // r0
  unsigned __int16 *v12; // r4
  unsigned int v13; // r0
  _WORD *v14; // r6
  unsigned __int16 v15; // t1
  unsigned __int16 v16; // r2
  unsigned int v17; // r1
  int v18; // r0

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
  v12 = (unsigned __int16 *)(a3 + 136);
  v13 = sub_16C9C((unsigned __int16 *)(a3 + 136), 1);
  v15 = *(_WORD *)(a3 + 140);
  v14 = (_WORD *)(a3 + 140);
  v16 = v15;
  v17 = (unsigned __int16)((v13 >> 1) - *(_WORD *)(a2 + 40));
  if ( v17 < v15 )
    *v14 = v16 - v17;
  else
    *v14 = 0;
  v18 = sub_16C9C(v12, 0);
  sub_16D78(v12, ((unsigned int)(v18 + 1) >> 1) - *(_WORD *)(a2 + 38));
  if ( *(_DWORD *)(a2 + 28) == *(_DWORD *)(a1 + 8) )
    return 1;
  return v10;
}

