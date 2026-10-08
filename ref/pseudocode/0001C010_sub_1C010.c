/* ea:   0x0001C010 */
/* name: sub_1C010 */
/* size: 1300 */
unsigned int __fastcall sub_1C010(int a1, int a2)
{
  unsigned int result; // r0
  __int16 v4; // r5
  unsigned int v5; // r3
  unsigned __int16 v6; // r2
  __int16 v7; // r4
  int v8; // r2
  int v9; // r3
  unsigned int v10; // r4
  __int16 v11; // r3
  unsigned int v12; // r3
  unsigned __int16 v13; // r3
  unsigned int v14; // r4
  unsigned int v15; // r3
  int v16; // r4
  unsigned int v17; // r3
  int v18; // r3
  unsigned int v19; // r5
  __int16 v20; // r1
  int v21; // r3

  result = *(_DWORD *)(a1 + 1052);
  if ( a2 != 0 )
  {
    if ( result == 1 )
    {
      *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 402);
      v7 = *(_WORD *)(a1 + 1036);
      v8 = *(unsigned __int16 *)(a1 + 1044) << 16;
      if ( v7 != 0 )
      {
        result = *(unsigned __int16 *)(a1 + 1044);
        if ( result < 0x3E7 )
        {
          *(_WORD *)(a1 + 1044) = result + 1;
          v9 = (unsigned __int16)(v7 - 1);
          *(_WORD *)(a1 + 1036) = v9;
          result = v9 << 16;
          v10 = *(unsigned __int16 *)(a1 + 1038);
          if ( v10 > (unsigned __int16)v9 )
          {
            ++*(_WORD *)(a1 + 1046);
            result = (unsigned __int16)(v10 - 1);
            *(_WORD *)(a1 + 1038) = result;
          }
        }
      }
      else
      {
        *(_WORD *)(a1 + 1044) = 0;
        *(_WORD *)(a1 + 1036) = HIWORD(v8);
        v11 = *(_WORD *)(a1 + 1046);
        *(_WORD *)(a1 + 1046) = 0;
        *(_WORD *)(a1 + 1038) += v11;
        return 1038;
      }
    }
    else if ( result == 3 )
    {
      *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 802);
      v4 = *(_WORD *)(a1 + 1038);
      if ( v4 != 0 )
      {
        result = a1 + 1024;
        v5 = *(unsigned __int16 *)(a1 + 1046);
        if ( v5 < *(unsigned __int16 *)(a1 + 1044) )
        {
          *(_WORD *)(a1 + 1046) = v5 + 1;
          result = (unsigned __int16)(v4 - 1);
          *(_WORD *)(a1 + 1038) = result;
        }
      }
      else
      {
        v6 = *(_WORD *)(a1 + 1036);
        if ( *(unsigned __int16 *)(a1 + 1046) <= (unsigned int)v6 )
          v6 = *(_WORD *)(a1 + 1046);
        *(_WORD *)(a1 + 1046) -= v6;
        *(_WORD *)(a1 + 1038) = v4 + v6;
        return 1038;
      }
    }
  }
  else if ( result == 1 )
  {
    *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 302);
    v15 = *(unsigned __int16 *)(a1 + 1036);
    if ( *(_WORD *)(a1 + 1044) != 0 )
    {
      result = 999;
      if ( v15 < 0x3E7 )
      {
        v16 = (unsigned __int16)(*(_WORD *)(a1 + 1044) - 1);
        *(_WORD *)(a1 + 1044) = v16;
        *(_WORD *)(a1 + 1036) = v15 + 1;
        result = v16 << 16;
        v17 = *(unsigned __int16 *)(a1 + 1046);
        if ( v17 > (unsigned __int16)v16 )
        {
          *(_WORD *)(a1 + 1046) = v17 - 1;
          result = (unsigned __int16)(*(_WORD *)(a1 + 1038) + 1);
          *(_WORD *)(a1 + 1038) = result;
        }
      }
    }
    else
    {
      *(_WORD *)(a1 + 1044) = v15;
      v18 = (unsigned __int16)(*(_WORD *)(a1 + 1036) - v15);
      *(_WORD *)(a1 + 1036) = v18;
      result = v18 << 16;
      v19 = *(unsigned __int16 *)(a1 + 1038);
      if ( (unsigned __int16)v18 < v19 )
      {
        v20 = v18;
        v21 = v19 - (unsigned __int16)v18 + *(unsigned __int16 *)(a1 + 1046);
        if ( v21 > *(unsigned __int16 *)(a1 + 1044) )
          *(_WORD *)(a1 + 1046) = *(_WORD *)(a1 + 1044);
        else
          *(_WORD *)(a1 + 1046) = v21;
        *(_WORD *)(a1 + 1038) = v20;
        return 1038;
      }
    }
  }
  else if ( result == 3 )
  {
    *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 702);
    if ( *(_WORD *)(a1 + 1046) != 0 )
    {
      result = a1 + 1024;
      v12 = *(unsigned __int16 *)(a1 + 1038);
      if ( v12 < *(unsigned __int16 *)(a1 + 1036) )
      {
        --*(_WORD *)(a1 + 1046);
        *(_WORD *)(a1 + 1038) = v12 + 1;
        return 1038;
      }
    }
    else
    {
      v13 = *(_WORD *)(a1 + 1044);
      v14 = *(unsigned __int16 *)(a1 + 1038);
      if ( v14 <= v13 )
        v13 = *(_WORD *)(a1 + 1038);
      *(_WORD *)(a1 + 1046) = v13;
      *(_WORD *)(a1 + 1038) = v14 - v13;
      return 1038;
    }
  }
  return result;
}

