/* ea:   0x00055E14 */
/* name: sub_55E14 */
/* size: 804 */
int __fastcall sub_55E14(int a1, int a2)
{
  unsigned __int16 v5; // r9
  unsigned __int16 *v6; // r10
  int v7; // r4
  unsigned __int16 v8; // r8
  unsigned __int16 v9; // r10
  unsigned __int8 v10; // r11
  unsigned __int8 v11; // r2
  unsigned __int8 v12; // r0
  unsigned int v13; // r6
  int v14; // r5
  unsigned __int16 v15; // r4
  int v16; // r9
  unsigned int v17; // r0
  int v18; // r0
  __int16 v19; // r0
  __int16 v20; // r0
  unsigned __int16 v21; // r0
  unsigned __int16 v22; // r0
  unsigned int v23; // r0
  int v24; // r0
  int v25; // [sp+0h] [bp-30h]

  if ( a2 == 0 )
    return 0;
  v5 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v6 = (unsigned __int16 *)(a2 + 136);
  v25 = 10 * v5;
  v7 = 10 * (unsigned __int16)((unsigned int)(sub_16C9C((unsigned __int16 *)(a2 + 136), 0) + 1) >> 1);
  v8 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
  v9 = (unsigned int)sub_16C9C(v6, 1) >> 1;
  v10 = 1;
  v11 = 2;
  v12 = 2;
  if ( *(_DWORD *)(a1 + 176) != 8 )
    v12 = 1;
  if ( *(_DWORD *)(a2 + 176) != 8 )
    v11 = 1;
  v13 = v8;
  v14 = v7 - 2 * (v8 + v5) * v12;
  if ( v14 <= 0 )
  {
LABEL_18:
    v17 = sub_16C9C((unsigned __int16 *)(a2 + 136), 1);
    sub_357C0((_WORD *)(a2 + 140), v17 >> 1);
    v18 = sub_16C9C((unsigned __int16 *)(a2 + 136), 0);
    sub_16D78((unsigned __int16 *)(a2 + 136), (unsigned int)(v18 + 1) >> 1);
    v19 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
    sub_357C0((_WORD *)(a1 + 140), v19 - v8);
    v20 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
    sub_16D78((unsigned __int16 *)(a1 + 136), v20 - v5);
    v16 = v25;
  }
  else
  {
    while ( 1 )
    {
      v15 = v14 / 10 + 1;
      if ( v15 > 0x3E7u )
        v15 = 999;
      if ( v15 <= (unsigned int)v9 )
        v9 = v15;
      v16 = v25 - 2 * (v15 + v9) * v11;
      v25 = v16;
      if ( v16 <= 0 )
        break;
      v5 = v16 / 10 + 1;
      if ( v5 > 0x3E7u )
        v5 = 999;
      if ( v13 >= v5 )
        v8 = v5;
      v13 = v8;
      v14 -= 2 * (v8 + v5) * v12;
      if ( v14 <= 0 )
        goto LABEL_18;
    }
    v21 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
    sub_357C0((_WORD *)(a1 + 140), v21);
    v22 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
    sub_16D78((unsigned __int16 *)(a1 + 136), v22);
    v23 = sub_16C9C((unsigned __int16 *)(a2 + 136), 1);
    sub_357C0((_WORD *)(a2 + 140), (v23 >> 1) - v9);
    v24 = sub_16C9C((unsigned __int16 *)(a2 + 136), 0);
    sub_16D78((unsigned __int16 *)(a2 + 136), ((unsigned int)(v24 + 1) >> 1) - v15);
  }
  if ( v16 <= v14 )
    return 0;
  return v10;
}

