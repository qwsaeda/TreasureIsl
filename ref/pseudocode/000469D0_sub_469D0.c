/* ea:   0x000469D0 */
/* name: sub_469D0 */
/* size: 812 */
BOOL __fastcall sub_469D0(int a1, int a2)
{
  unsigned __int16 *v3; // r6
  unsigned __int16 *v4; // r7
  unsigned __int16 v5; // r9
  int v6; // r4
  unsigned __int16 v7; // r8
  unsigned __int16 v8; // r10
  unsigned __int8 v9; // r2
  unsigned __int8 v10; // r0
  unsigned int v11; // r6
  int v12; // r5
  unsigned __int16 v13; // r4
  int v14; // r9
  unsigned int v15; // r0
  int v16; // r0
  unsigned int v17; // r0
  unsigned int v18; // r0
  unsigned int v19; // r0
  int v20; // r0
  unsigned int v21; // r0
  int v22; // r0
  int v24; // [sp+0h] [bp-30h]

  v3 = (unsigned __int16 *)(a1 + 136);
  v4 = (unsigned __int16 *)(a2 + 136);
  v5 = (unsigned int)(sub_16C9C((unsigned __int16 *)(a1 + 136), 0) + 1) >> 1;
  v24 = 10 * v5;
  v6 = 10 * (unsigned __int16)((unsigned int)(sub_16C9C(v4, 0) + 1) >> 1);
  v7 = (unsigned int)sub_16C9C(v3, 1) >> 1;
  v8 = (unsigned int)sub_16C9C(v4, 1) >> 1;
  v9 = 2;
  v10 = 2;
  if ( *(_DWORD *)(a1 + 176) != 8 )
    v10 = 1;
  if ( *(_DWORD *)(a2 + 176) != 8 )
    v9 = 1;
  v11 = v7;
  v12 = v6 - 2 * (v7 + v5) * v10;
  if ( v12 <= 0 )
  {
LABEL_16:
    v15 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
    sub_357C0((_WORD *)(a2 + 140), v15 >> 1);
    v16 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
    sub_16D78(v4, (unsigned int)(v16 + 1) >> 1);
    v17 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
    sub_357C0((_WORD *)(a1 + 140), (v17 >> 1) - v7);
    v18 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
    sub_16D78((unsigned __int16 *)(a1 + 136), (v18 >> 1) - v5 + 1);
    v14 = v24;
  }
  else
  {
    while ( 1 )
    {
      v13 = v12 / 10 + 1;
      if ( v13 > 0x3E7u )
        v13 = 999;
      if ( v13 <= (unsigned int)v8 )
        v8 = v13;
      v14 = v24 - 2 * (v13 + v8) * v9;
      v24 = v14;
      if ( v14 <= 0 )
        break;
      v5 = v14 / 10 + 1;
      if ( v5 > 0x3E7u )
        v5 = 999;
      if ( v11 >= v5 )
        v7 = v5;
      v11 = v7;
      v12 -= 2 * (v7 + v5) * v10;
      if ( v12 <= 0 )
        goto LABEL_16;
    }
    v19 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
    sub_357C0((_WORD *)(a1 + 140), v19 >> 1);
    v20 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
    sub_16D78((unsigned __int16 *)(a1 + 136), (unsigned int)(v20 + 1) >> 1);
    v21 = sub_16C9C(v4, 1);
    sub_357C0((_WORD *)(a2 + 140), (v21 >> 1) - v8);
    v22 = sub_16C9C(v4, 0);
    sub_16D78(v4, ((unsigned int)(v22 + 1) >> 1) - v13);
  }
  return v14 > v12;
}

