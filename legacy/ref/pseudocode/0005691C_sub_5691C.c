/* ea:   0x0005691C */
/* name: sub_5691C */
/* size: 260 */
_DWORD **__fastcall sub_5691C(int a1, int a2, char a3)
{
  unsigned int v5; // r1
  unsigned int v6; // r0
  _DWORD *v7; // r4
  unsigned int v8; // t1
  unsigned int v9; // r2
  int v10; // r3
  unsigned int v11; // r1
  int v12; // r2
  unsigned __int16 v13; // r0
  int v14; // r4
  unsigned int v15; // r1
  int v16; // r6
  unsigned __int16 v17; // r0
  int v18; // r4
  unsigned int v19; // r0
  _DWORD **result; // r0

  if ( a2 != 0 && a3 != 0 )
  {
    v5 = sub_16C9C((unsigned __int16 *)(a2 + 136), 2) + 3;
    v8 = *(_DWORD *)(a2 + 144);
    v7 = (_DWORD *)(a2 + 144);
    v6 = v8;
    v9 = v5 >> 2;
    if ( v5 >> 2 >= v8 )
      *v7 = 0;
    else
      *v7 = v6 - v9;
    v10 = *(_DWORD *)(a1 + 16);
    v11 = *(_DWORD *)(v10 + 8) + v9;
    v12 = 999999;
    if ( v11 <= 0xF423F )
      v12 = v11;
    *(_DWORD *)(v10 + 8) = v12;
    v13 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
    v14 = *(_DWORD *)(a1 + 16);
    v15 = *(unsigned __int16 *)(v14 + 12) + v13;
    LOWORD(v16) = 999;
    if ( v15 > 0x3E7 )
      LOWORD(v15) = 999;
    *(_WORD *)(v14 + 12) = v15;
    v17 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
    v18 = *(_DWORD *)(a1 + 16);
    v19 = v17 << 16;
    if ( *(unsigned __int16 *)(v18 + 14) + HIWORD(v19) <= 0x3E7u )
      v16 = *(unsigned __int16 *)(v18 + 14) + HIWORD(v19);
    *(_WORD *)(v18 + 14) = v16;
  }
  result = sub_52CE8(a1 + 184);
  *(_BYTE *)(a1 + 4) = 0;
  return result;
}

