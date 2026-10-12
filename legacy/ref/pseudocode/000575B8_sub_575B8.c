/* ea:   0x000575B8 */
/* name: sub_575B8 */
/* size: 1004 */
int __fastcall sub_575B8(int a1)
{
  int v2; // r1
  int v3; // r0
  int v4; // r0
  int v5; // r0
  int v6; // r0
  int v7; // r0
  int v8; // r0
  int v9; // r0
  unsigned __int16 v10; // r1
  int v11; // r4
  int result; // r0
  __int16 v13; // r3
  unsigned __int16 v14; // r4
  unsigned __int16 v15; // r1
  unsigned int v16; // r0
  int v17; // r1
  unsigned int v18; // r2
  int v19; // r3
  __int16 v20; // r4
  unsigned __int16 v21; // r4

  if ( *(_DWORD *)(a1 + 1136) == 36 )
  {
    sub_17C7C((int)off_811A0, 2);
    *(_DWORD *)(a1 + 1136) = 0;
  }
  v2 = *(_DWORD *)(*(_DWORD *)(a1 + 1100) + 4);
  if ( *(_DWORD *)(a1 + 1104) != 0 )
  {
    if ( v2 != 0 )
      v3 = *(_DWORD *)(a1 + 1032);
    else
      v3 = *(_DWORD *)(a1 + 1036);
    *(_DWORD *)(a1 + 1108) = v3;
    sub_462A4(v3, 0);
    *(_DWORD *)(*(_DWORD *)(a1 + 1108) + 12) = 20;
    *(_DWORD *)(*(_DWORD *)(a1 + 1108) + 16) = 95;
    *(_DWORD *)(*(_DWORD *)(a1 + 1108) + 28) = 1;
    if ( *(_DWORD *)(*(_DWORD *)(a1 + 1100) + 20) != 0 )
    {
      v4 = *(_DWORD *)(a1 + 1040);
      *(_DWORD *)(a1 + 1112) = v4;
      sub_462A4(v4, 0);
      v5 = 121;
    }
    else
    {
      v6 = *(_DWORD *)(a1 + 1044);
      *(_DWORD *)(a1 + 1112) = v6;
      sub_462A4(v6, 0);
      v5 = 109;
    }
  }
  else
  {
    if ( v2 != 0 )
      v7 = *(_DWORD *)(a1 + 1040);
    else
      v7 = *(_DWORD *)(a1 + 1044);
    *(_DWORD *)(a1 + 1108) = v7;
    sub_462A4(v7, 0);
    *(_DWORD *)(*(_DWORD *)(a1 + 1108) + 12) = 20;
    *(_DWORD *)(*(_DWORD *)(a1 + 1108) + 16) = 95;
    *(_DWORD *)(*(_DWORD *)(a1 + 1108) + 28) = 1;
    if ( *(_DWORD *)(*(_DWORD *)(a1 + 1100) + 20) != 0 )
    {
      v8 = *(_DWORD *)(a1 + 1032);
      *(_DWORD *)(a1 + 1112) = v8;
      sub_462A4(v8, 0);
      v5 = 112;
    }
    else
    {
      v9 = *(_DWORD *)(a1 + 1036);
      *(_DWORD *)(a1 + 1112) = v9;
      sub_462A4(v9, 0);
      v5 = 56;
    }
  }
  *(_DWORD *)(*(_DWORD *)(a1 + 1112) + 12) = v5;
  *(_DWORD *)(*(_DWORD *)(a1 + 1112) + 16) = 95;
  *(_DWORD *)(*(_DWORD *)(a1 + 1112) + 28) = 3;
  sub_45F30(
    *(_DWORD **)(a1 + 1108),
    *(_DWORD *)(*(_DWORD *)(a1 + 1100) + 8),
    *(int **)(a1 + 4),
    *(int (__fastcall **)(int, int, int, int, int))(a1 + 488));
  sub_45F30(
    *(_DWORD **)(a1 + 1112),
    *(_DWORD *)(*(_DWORD *)(a1 + 1100) + 16),
    *(int **)(a1 + 4),
    *(int (__fastcall **)(int, int, int, int, int))(a1 + 488));
  if ( *(_DWORD *)(*(_DWORD *)(a1 + 1112) + 32) != 0 && *(_DWORD *)(*(_DWORD *)(a1 + 1108) + 32) != 0 )
  {
    if ( *(_DWORD *)(a1 + 1104) != 0 )
    {
      v10 = *(_WORD *)(a1 + 1086);
      *(_DWORD *)(a1 + 1104) = 0;
      v11 = *(_DWORD *)(a1 + 1124)
          - 2
          * (v10 + *(unsigned __int16 *)(a1 + 1084))
          * (100 - *(unsigned __int16 *)(a1 + 1096))
          * *(unsigned __int16 *)(a1 + 1088)
          / 100;
      *(_DWORD *)(a1 + 1124) = v11;
      if ( v11 > 0 )
      {
        v13 = v11 / 10;
        *(_WORD *)(a1 + 1090) = v13;
        if ( v11 % 10 != 0 )
          *(_WORD *)(a1 + 1090) = v13 + 1;
        v14 = *(_WORD *)(a1 + 1092);
        if ( v14 >= (unsigned int)*(unsigned __int16 *)(a1 + 1090) )
          v14 = *(_WORD *)(a1 + 1090);
        *(_WORD *)(a1 + 1092) = v14;
        return a1 + 1024;
      }
      else
      {
        *(_DWORD *)(a1 + 1024) = 1;
        *(_WORD *)(a1 + 1090) = 0;
        result = *(_DWORD *)(a1 + 1056);
        *(_WORD *)(a1 + 1092) = 0;
        *(_DWORD *)(a1 + 1080) = result;
      }
    }
    else
    {
      v15 = *(_WORD *)(a1 + 1092);
      *(_DWORD *)(a1 + 1104) = 1;
      v16 = (int)((unsigned __int64)(2748779070LL
                                   * (v15 + *(unsigned __int16 *)(a1 + 1090))
                                   * *(unsigned __int16 *)(a1 + 1094)) >> 32) >> 5;
      v17 = *(_DWORD *)(a1 + 1128);
      v18 = v16 + (v16 >> 31);
      *(_DWORD *)(a1 + 1128) = v17 - v18;
      if ( (int)(v17 - v18) > 0 )
      {
        v19 = *(_DWORD *)(a1 + 1128);
        v20 = (int)(v17 - v18) / 10;
        *(_WORD *)(a1 + 1084) = v20;
        if ( v19 % 10 != 0 )
          *(_WORD *)(a1 + 1084) = v20 + 1;
        v21 = *(_WORD *)(a1 + 1086);
        if ( v21 >= (unsigned int)*(unsigned __int16 *)(a1 + 1084) )
          v21 = *(_WORD *)(a1 + 1084);
        *(_WORD *)(a1 + 1086) = v21;
        return 1086;
      }
      else
      {
        *(_DWORD *)(a1 + 1024) = 1;
        *(_WORD *)(a1 + 1084) = 0;
        result = *(_DWORD *)(a1 + 1072);
        *(_WORD *)(a1 + 1086) = 0;
        *(_DWORD *)(a1 + 1080) = result;
      }
    }
  }
  else
  {
    result = *(_DWORD *)(a1 + 1136);
    *(_DWORD *)(a1 + 1136) = result + 1;
  }
  return result;
}

