/* ea:   0x00046FD8 */
/* name: sub_46FD8 */
/* size: 888 */
BOOL __fastcall sub_46FD8(int a1, int a2)
{
  unsigned __int16 *v4; // r7
  unsigned __int16 v5; // r11
  int v6; // r6
  unsigned __int16 v7; // r10
  unsigned __int16 v8; // r8
  unsigned __int16 v9; // r3
  unsigned __int8 v10; // r0
  unsigned int v11; // r4
  int v12; // r5
  unsigned __int16 v13; // r6
  unsigned __int16 v14; // r11
  int v15; // r2
  unsigned int v16; // r0
  int v17; // r0
  unsigned int v18; // r0
  int v19; // r0
  unsigned __int16 v20; // [sp+0h] [bp-38h]
  int v22; // [sp+8h] [bp-30h]

  if ( a2 == 0 )
    return false;
  v4 = (unsigned __int16 *)(a1 + 136);
  v5 = (unsigned int)(sub_16C9C((unsigned __int16 *)(a1 + 136), 0) + 1) >> 1;
  v20 = *(_WORD *)(a2 + 12);
  v22 = 10 * v5;
  v6 = 10 * v20;
  v7 = (unsigned int)sub_16C9C(v4, 0) >> 1;
  v8 = *(_WORD *)(a2 + 14);
  if ( v8 >= (unsigned int)v20 )
  {
    v9 = v20;
  }
  else
  {
    v9 = v8;
    v20 = v8;
  }
  v10 = 2;
  if ( *(_DWORD *)(a1 + 176) != 8 )
    v10 = 1;
  v11 = v7;
  v12 = v6 - 40 * (v7 + v5) * v10 / 100;
  if ( v12 <= 0 )
  {
LABEL_22:
    *(_WORD *)(a2 + 14) = 0;
    *(_WORD *)(a2 + 12) = 0;
    v16 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
    sub_357C0((_WORD *)(a1 + 140), (v16 >> 1) - v7);
    v17 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
    sub_16D78((unsigned __int16 *)(a1 + 136), ((unsigned int)(v17 + 1) >> 1) - v5);
  }
  else
  {
    while ( 1 )
    {
      v13 = v12 / 10 + 1;
      if ( v13 > 0x3E7u )
        v13 = 999;
      if ( v13 > (unsigned int)v9 )
      {
        v14 = v20;
      }
      else
      {
        v14 = v13;
        v20 = v13;
      }
      v15 = v22 - 1000 * (v13 + (unsigned int)v14) / 0x64;
      v22 = v15;
      if ( v15 <= 0 )
        break;
      v5 = v15 / 10 + 1;
      if ( v5 > 0x3E7u )
        v5 = 999;
      if ( v11 >= v5 )
        v7 = v5;
      v11 = v7;
      v12 -= 40 * (v7 + v5) * v10 / 100;
      if ( v12 <= 0 )
        goto LABEL_22;
      v9 = v20;
    }
    v18 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
    sub_357C0((_WORD *)(a1 + 140), v18 >> 1);
    v19 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
    sub_16D78((unsigned __int16 *)(a1 + 136), (unsigned int)(v19 + 1) >> 1);
    sub_4A758(a2, (unsigned __int16)(*(_WORD *)(a2 + 14) - v14));
    sub_4A71C(a2, (unsigned __int16)(*(_WORD *)(a2 + 12) - v13));
  }
  return v22 > v12;
}

