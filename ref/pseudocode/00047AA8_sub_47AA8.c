/* ea:   0x00047AA8 */
/* name: sub_47AA8 */
/* size: 652 */
BOOL __fastcall sub_47AA8(int a1)
{
  unsigned __int16 *v2; // r5
  int v3; // r11
  int v4; // r4
  unsigned __int16 v5; // r9
  unsigned __int16 v6; // r0
  unsigned __int16 v7; // r5
  unsigned __int16 v8; // r8
  unsigned __int8 v9; // r0
  unsigned int v10; // r6
  int v11; // r4
  unsigned int v12; // r2
  unsigned __int16 *v13; // r5
  unsigned __int16 v14; // r1
  _WORD *v15; // r10
  unsigned __int16 v16; // t1
  unsigned __int16 v17; // r2
  unsigned __int16 v18; // r1
  unsigned __int16 v19; // r0
  unsigned __int16 *v21; // [sp+0h] [bp-28h]

  v2 = (unsigned __int16 *)(a1 + 136);
  v21 = (unsigned __int16 *)(a1 + 136);
  v3 = 10 * sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v4 = 10 * word_6FE24[dword_7FB6C];
  v5 = sub_16C9C(v2, 0);
  v6 = sub_16C9C(v2, 1);
  v7 = 0;
  v8 = v6;
  v9 = 2;
  if ( *(_DWORD *)(a1 + 176) != 8 )
    v9 = 1;
  v10 = v8;
  v11 = v4 - 40 * (v8 + v5) * (unsigned int)v9 / 0x64;
  if ( v11 <= 0 )
  {
LABEL_10:
    v13 = v21;
    v14 = sub_16C9C(v21, 1) - v8;
    v16 = *(_WORD *)(a1 + 140);
    v15 = (_WORD *)(a1 + 140);
    v17 = v16;
    if ( v14 < (unsigned int)v16 )
      *v15 = v17 - v14;
    else
      *v15 = 0;
    v18 = sub_16C9C(v21, 0) - v5;
  }
  else
  {
    while ( 1 )
    {
      v12 = (unsigned __int16)(v11 / 10 + 1);
      if ( v12 <= v7 )
        v7 = v11 / 10 + 1;
      v3 -= (int)(1000 * (v12 + v7)) / 100;
      if ( v3 <= 0 )
        break;
      v5 = v3 / 10 + 1;
      if ( v10 >= v5 )
        v8 = v3 / 10 + 1;
      v10 = v8;
      v11 -= 40 * ((unsigned __int16)(v3 / 10 + 1) + v8) * v9 / 100;
      if ( v11 <= 0 )
        goto LABEL_10;
    }
    v13 = v21;
    v19 = sub_16C9C(v21, 1);
    sub_357C0((_WORD *)(a1 + 140), v19);
    v18 = sub_16C9C(v21, 0);
  }
  sub_16D78(v13, v18);
  return v3 > v11;
}

