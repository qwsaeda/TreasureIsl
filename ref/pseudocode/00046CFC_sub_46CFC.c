/* ea:   0x00046CFC */
/* name: sub_46CFC */
/* size: 728 */
BOOL __fastcall sub_46CFC(int a1, int a2)
{
  unsigned __int16 *v3; // r11
  unsigned __int16 *v4; // r6
  unsigned __int16 v5; // r8
  int v6; // r4
  unsigned __int16 v7; // r7
  unsigned __int16 v8; // r9
  unsigned __int8 v9; // r0
  unsigned int v10; // r3
  int v11; // r5
  unsigned __int16 v12; // r4
  int v13; // r2
  unsigned __int16 v14; // r0
  unsigned __int16 v15; // r0
  unsigned int v16; // r0
  unsigned int v17; // r1
  unsigned __int16 *v18; // r0
  unsigned int v19; // r0
  int v20; // r0
  __int16 v21; // r0
  int v23; // [sp+0h] [bp-2Ch]

  v3 = (unsigned __int16 *)(a1 + 136);
  v4 = (unsigned __int16 *)(a2 + 136);
  v5 = (unsigned int)(sub_16C9C((unsigned __int16 *)(a1 + 136), 0) + 1) >> 1;
  v23 = 10 * v5;
  v6 = 10 * (unsigned __int16)sub_16C9C(v4, 0);
  v7 = (unsigned int)sub_16C9C(v3, 1) >> 1;
  v8 = sub_16C9C(v4, 1);
  v9 = 2;
  if ( *(_DWORD *)(a1 + 176) != 8 )
    v9 = 1;
  v10 = v7;
  v11 = v6 - 2 * (v7 + v5) * v9;
  if ( v11 <= 0 )
  {
LABEL_14:
    v14 = sub_16C9C((unsigned __int16 *)(a2 + 136), 1);
    sub_357C0((_WORD *)(a2 + 140), v14);
    v15 = sub_16C9C((unsigned __int16 *)(a2 + 136), 0);
    sub_16D78((unsigned __int16 *)(a2 + 136), v15);
    v16 = sub_16C9C(v3, 1);
    sub_357C0((_WORD *)(a1 + 140), (v16 >> 1) - v7);
    v17 = ((unsigned int)(sub_16C9C(v3, 0) + 1) >> 1) - v5;
    v18 = v3;
  }
  else
  {
    while ( 1 )
    {
      v12 = v11 / 10 + 1;
      if ( v12 > 0x3E7u )
        v12 = 999;
      if ( v12 <= (unsigned int)v8 )
        v8 = v12;
      v13 = v23 - 2 * (v12 + v8);
      v23 = v13;
      if ( v13 <= 0 )
        break;
      v5 = v13 / 10 + 1;
      if ( v5 > 0x3E7u )
        v5 = 999;
      if ( v10 >= v5 )
        v7 = v5;
      v10 = v7;
      v11 -= 2 * (v7 + v5) * v9;
      if ( v11 <= 0 )
        goto LABEL_14;
    }
    v19 = sub_16C9C(v3, 1);
    sub_357C0((_WORD *)(a1 + 140), v19 >> 1);
    v20 = sub_16C9C(v3, 0);
    sub_16D78(v3, (unsigned int)(v20 + 1) >> 1);
    v21 = sub_16C9C((unsigned __int16 *)(a2 + 136), 1);
    sub_357C0((_WORD *)(a2 + 140), v21 - v8);
    LOWORD(v17) = sub_16C9C((unsigned __int16 *)(a2 + 136), 0) - v12;
    v18 = (unsigned __int16 *)(a2 + 136);
  }
  sub_16D78(v18, v17);
  return v23 > v11;
}

