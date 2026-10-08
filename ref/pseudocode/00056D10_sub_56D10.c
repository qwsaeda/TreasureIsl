/* ea:   0x00056D10 */
/* name: sub_56D10 */
/* size: 248 */
_DWORD **__fastcall sub_56D10(int a1, int a2, char a3)
{
  unsigned int v5; // r1
  unsigned int v6; // r2
  unsigned int v7; // r0
  int v8; // r3
  unsigned int v9; // r1
  int v10; // r2
  unsigned __int16 v11; // r0
  int v12; // r4
  unsigned int v13; // r1
  int v14; // r6
  unsigned __int16 v15; // r0
  int v16; // r4
  unsigned int v17; // r0
  _DWORD **result; // r0

  if ( a2 != 0 && a3 != 0 )
  {
    v5 = *(_DWORD *)(a2 + 8);
    v6 = (v5 + 3) >> 2;
    v7 = 0;
    if ( v6 <= v5 )
      v7 = v5 - v6;
    *(_DWORD *)(a2 + 8) = v7;
    v8 = *(_DWORD *)(a1 + 16);
    v9 = *(_DWORD *)(v8 + 8) + v6;
    v10 = 999999;
    if ( v9 <= 0xF423F )
      v10 = v9;
    *(_DWORD *)(v8 + 8) = v10;
    v11 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
    v12 = *(_DWORD *)(a1 + 16);
    v13 = *(unsigned __int16 *)(v12 + 12) + v11;
    LOWORD(v14) = 999;
    if ( v13 > 0x3E7 )
      LOWORD(v13) = 999;
    *(_WORD *)(v12 + 12) = v13;
    v15 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
    v16 = *(_DWORD *)(a1 + 16);
    v17 = v15 << 16;
    if ( *(unsigned __int16 *)(v16 + 14) + HIWORD(v17) <= 0x3E7u )
      v14 = *(unsigned __int16 *)(v16 + 14) + HIWORD(v17);
    *(_WORD *)(v16 + 14) = v14;
  }
  result = sub_52CE8(a1 + 184);
  *(_BYTE *)(a1 + 4) = 0;
  return result;
}

