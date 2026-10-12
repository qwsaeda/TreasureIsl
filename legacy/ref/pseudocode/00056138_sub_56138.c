/* ea:   0x00056138 */
/* name: sub_56138 */
/* size: 860 */
BOOL __fastcall sub_56138(int a1, int a2)
{
  unsigned __int16 v5; // r11
  int v6; // r6
  unsigned __int16 v7; // r10
  unsigned __int8 v8; // r0
  unsigned __int16 v9; // r3
  unsigned int v10; // r4
  int v11; // r5
  unsigned __int16 v12; // r6
  unsigned __int16 v13; // r11
  int v14; // r2
  __int16 v15; // r0
  __int16 v16; // r0
  unsigned __int16 v17; // r0
  unsigned __int16 v18; // r0
  unsigned __int16 v19; // [sp+0h] [bp-38h]
  int v21; // [sp+8h] [bp-30h]

  if ( a2 == 0 )
    return false;
  v5 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v19 = *(_WORD *)(a2 + 12);
  v21 = 10 * v5;
  v6 = 10 * v19;
  v7 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
  v8 = 2;
  if ( *(_DWORD *)(a1 + 176) != 8 )
    v8 = 1;
  if ( *(unsigned __int16 *)(a2 + 14) >= (unsigned int)v19 )
  {
    v9 = v19;
  }
  else
  {
    v9 = *(_WORD *)(a2 + 14);
    v19 = v9;
  }
  v10 = v7;
  v11 = v6 - 40 * (v7 + v5) * v8 / 100;
  if ( v11 <= 0 )
  {
LABEL_22:
    *(_WORD *)(a2 + 14) = 0;
    *(_WORD *)(a2 + 12) = 0;
    v15 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
    sub_357C0((_WORD *)(a1 + 140), v15 - v7);
    v16 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
    sub_16D78((unsigned __int16 *)(a1 + 136), v16 - v5);
  }
  else
  {
    while ( 1 )
    {
      v12 = v11 / 10 + 1;
      if ( v12 > 0x3E7u )
        v12 = 999;
      if ( v12 > (unsigned int)v9 )
      {
        v13 = v19;
      }
      else
      {
        v13 = v12;
        v19 = v12;
      }
      v14 = v21 - 1000 * (v12 + (unsigned int)v13) / 0x64;
      v21 = v14;
      if ( v14 <= 0 )
        break;
      v5 = v14 / 10 + 1;
      if ( v5 > 0x3E7u )
        v5 = 999;
      if ( v10 >= v5 )
        v7 = v5;
      v10 = v7;
      v11 -= 40 * (v7 + v5) * v8 / 100;
      if ( v11 <= 0 )
        goto LABEL_22;
      v9 = v19;
    }
    v17 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
    sub_357C0((_WORD *)(a1 + 140), v17);
    v18 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
    sub_16D78((unsigned __int16 *)(a1 + 136), v18);
    sub_4A758(a2, *(_WORD *)(a2 + 14) - v13);
    sub_4A71C(a2, *(_WORD *)(a2 + 12) - v12);
  }
  return v21 > v11;
}

