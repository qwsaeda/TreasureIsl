/* ea:   0x00047820 */
/* name: sub_47820 */
/* size: 648 */
BOOL __fastcall sub_47820(int a1, unsigned __int16 *a2)
{
  unsigned __int16 *v4; // r11
  unsigned __int16 v5; // r5
  int v6; // r4
  unsigned __int16 v7; // r6
  int v8; // r9
  int v9; // r5
  unsigned int v10; // r1
  int v11; // r0
  unsigned __int8 v12; // r0
  int v13; // r6
  unsigned __int16 v14; // r8
  unsigned __int16 v15; // r5
  unsigned __int16 v16; // r3
  unsigned int v17; // r1
  _WORD *v18; // r10
  unsigned __int16 v19; // t1
  unsigned __int16 v20; // r2
  unsigned __int16 v21; // r1
  unsigned int v22; // r0

  v4 = (unsigned __int16 *)(a1 + 136);
  v5 = (unsigned int)(sub_16C9C((unsigned __int16 *)(a1 + 136), 0) + 1) >> 1;
  v6 = sub_16C9C(v4, 0);
  v7 = (rand() % 30 + 1) * v6 / 0x64u;
  if ( v7 == 0 )
    v7 = 1;
  v8 = 10 * v7;
  v9 = 10 * v5;
  v10 = (unsigned int)sub_16C9C(v4, 1) >> 1;
  v11 = *(_DWORD *)(a1 + 176);
  *a2 = v7;
  if ( v11 == 8 )
    v12 = 2;
  else
    v12 = 1;
  v13 = v9 - 2 * v7;
  v14 = v10;
  if ( v13 <= 0 )
  {
LABEL_15:
    v17 = (unsigned int)sub_16C9C(v4, 1) >> 1;
    v19 = *(_WORD *)(a1 + 140);
    v18 = (_WORD *)(a1 + 140);
    v20 = v19;
    if ( (unsigned __int16)v17 < (unsigned int)v19 )
      *v18 = v20 - v17;
    else
      *v18 = 0;
    v21 = (unsigned int)(sub_16C9C(v4, 0) + 1) >> 1;
  }
  else
  {
    while ( 1 )
    {
      v15 = v13 / 10 + 1;
      if ( v15 > 0x3E7u )
        v15 = 999;
      if ( v15 <= (unsigned int)v14 )
        v14 = v15;
      v8 -= 2 * (v15 + v14) * v12;
      if ( v8 <= 0 )
        break;
      v16 = v8 / 10 + 1;
      if ( v16 > 0x3E7u )
        v16 = 999;
      v13 -= 2 * v16;
      if ( v13 <= 0 )
        goto LABEL_15;
    }
    v22 = sub_16C9C(v4, 1);
    sub_357C0((_WORD *)(a1 + 140), (v22 >> 1) - v14);
    v21 = ((unsigned int)(sub_16C9C(v4, 0) + 1) >> 1) - v15;
  }
  sub_16D78(v4, v21);
  return v13 > v8;
}

