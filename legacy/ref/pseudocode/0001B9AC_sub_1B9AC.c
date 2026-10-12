/* ea:   0x0001B9AC */
/* name: sub_1B9AC */
/* size: 1636 */
unsigned int __fastcall sub_1B9AC(int a1, int a2)
{
  unsigned int result; // r0
  __int16 v4; // r5
  unsigned __int16 v5; // r2
  int v6; // r0
  int v7; // r0
  int v8; // r0
  int v9; // r3
  int v10; // r2
  int v11; // r1
  int v12; // r1
  __int16 v13; // r1
  __int16 v14; // r4
  int v15; // r3
  unsigned int v16; // r4
  unsigned int v17; // r0
  unsigned int v18; // r3
  int v19; // r0
  int v20; // r0
  int v21; // r0
  int v22; // r3
  int v23; // r2
  int v24; // r1
  unsigned int v25; // r3
  int v26; // r3
  unsigned int v27; // r4
  int v28; // r1
  __int16 v29; // r3
  int v30; // r0

  result = *(_DWORD *)(a1 + 1052);
  if ( a2 == 0 )
  {
    if ( result == 1 )
    {
      *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 302);
      v25 = *(unsigned __int16 *)(a1 + 1036);
      if ( *(_WORD *)(a1 + 1044) != 0 )
      {
        result = 999;
        if ( v25 < 0x3E7 )
        {
          --*(_WORD *)(a1 + 1044);
          *(_WORD *)(a1 + 1036) = v25 + 1;
          return 1036;
        }
      }
      else
      {
        *(_WORD *)(a1 + 1044) = v25;
        v26 = (unsigned __int16)(*(_WORD *)(a1 + 1036) - v25);
        *(_WORD *)(a1 + 1036) = v26;
        result = v26 << 16;
        v27 = *(unsigned __int16 *)(a1 + 1038);
        v28 = (unsigned __int16)v26;
        if ( (unsigned __int16)v26 < v27 )
        {
          v29 = v27 - v26;
          v30 = v27 - v28 + *(unsigned __int16 *)(a1 + 1046);
          if ( v30 > 999 )
            *(_WORD *)(a1 + 1046) = 999;
          else
            *(_WORD *)(a1 + 1046) = v30;
          *(_WORD *)(a1 + 1038) = v27 - v29;
          return 1038;
        }
      }
      return result;
    }
    if ( result != 2 )
    {
      if ( result == 3 )
      {
        *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 702);
        v18 = *(unsigned __int16 *)(a1 + 1038);
        if ( *(_WORD *)(a1 + 1046) != 0 )
        {
          result = a1 + 1024;
          if ( v18 < *(unsigned __int16 *)(a1 + 1036) )
          {
            --*(_WORD *)(a1 + 1046);
            *(_WORD *)(a1 + 1038) = v18 + 1;
            return 1038;
          }
        }
        else
        {
          *(_WORD *)(a1 + 1046) = v18;
          *(_WORD *)(a1 + 1038) = 0;
          return 1038;
        }
      }
      return result;
    }
    *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 502);
    v19 = *(_DWORD *)(a1 + 1104);
    *(_DWORD *)(a1 + 1104) = v19 + 1;
    if ( v19 == 9 )
    {
      *(_DWORD *)(a1 + 1104) = 0;
      v20 = *(_DWORD *)(a1 + 1108);
      if ( v20 > 1000 )
      {
        if ( v20 != 10000 )
        {
          if ( v20 == 100000 )
            *(_DWORD *)(a1 + 1108) = 100000;
          goto LABEL_62;
        }
        v21 = 100000;
      }
      else
      {
        switch ( v20 )
        {
          case 1000:
            v21 = 10000;
            break;
          case 1:
            v21 = 10;
            break;
          case 10:
            v21 = 100;
            break;
          case 100:
            v21 = 1000;
            break;
          default:
            goto LABEL_62;
        }
      }
      *(_DWORD *)(a1 + 1108) = v21;
    }
LABEL_62:
    v22 = *(_DWORD *)(a1 + 1040);
    if ( v22 != 0 )
    {
      v23 = *(_DWORD *)(a1 + 1108);
      v24 = *(_DWORD *)(a1 + 1032);
      if ( v22 < v23 )
        v23 = *(_DWORD *)(a1 + 1040);
      if ( (unsigned int)(v24 + v23) > 0xF423F )
        v23 = 999999 - v24;
      *(_DWORD *)(a1 + 1040) = v22 - v23;
      *(_DWORD *)(a1 + 1032) = v24 + v23;
      return v22 - v23;
    }
    else
    {
      result = *(_DWORD *)(a1 + 1032);
      *(_DWORD *)(a1 + 1040) = result;
      *(_DWORD *)(a1 + 1032) = 0;
    }
    return result;
  }
  if ( result != 1 )
  {
    if ( result != 2 )
    {
      if ( result == 3 )
      {
        *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 802);
        v4 = *(_WORD *)(a1 + 1038);
        if ( v4 != 0 )
        {
          result = *(unsigned __int16 *)(a1 + 1046);
          if ( result < 0x3E7 )
          {
            *(_WORD *)(a1 + 1046) = result + 1;
            result = (unsigned __int16)(v4 - 1);
            *(_WORD *)(a1 + 1038) = result;
          }
        }
        else
        {
          v5 = *(_WORD *)(a1 + 1036);
          if ( *(unsigned __int16 *)(a1 + 1046) <= (unsigned int)v5 )
            v5 = *(_WORD *)(a1 + 1046);
          *(_WORD *)(a1 + 1046) -= v5;
          *(_WORD *)(a1 + 1038) = v4 + v5;
          return 1038;
        }
      }
      return result;
    }
    *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 602);
    v6 = *(_DWORD *)(a1 + 1104);
    *(_DWORD *)(a1 + 1104) = v6 + 1;
    if ( v6 != 9 )
      goto LABEL_26;
    *(_DWORD *)(a1 + 1104) = 0;
    v7 = *(_DWORD *)(a1 + 1108);
    if ( v7 > 1000 )
    {
      if ( v7 != 10000 )
      {
        if ( v7 == 100000 )
          *(_DWORD *)(a1 + 1108) = 100000;
        goto LABEL_26;
      }
      v8 = 100000;
    }
    else
    {
      switch ( v7 )
      {
        case 1000:
          v8 = 10000;
          break;
        case 1:
          v8 = 10;
          break;
        case 10:
          v8 = 100;
          break;
        case 100:
          v8 = 1000;
          break;
        default:
          goto LABEL_26;
      }
    }
    *(_DWORD *)(a1 + 1108) = v8;
LABEL_26:
    v9 = *(_DWORD *)(a1 + 1032);
    if ( v9 != 0 )
    {
      v10 = *(_DWORD *)(a1 + 1108);
      v11 = *(_DWORD *)(a1 + 1040);
      if ( v9 < v10 )
        v10 = *(_DWORD *)(a1 + 1032);
      if ( (unsigned int)(v11 + v10) > 0xF423F )
        v10 = 999999 - v11;
      *(_DWORD *)(a1 + 1040) = v11 + v10;
      *(_DWORD *)(a1 + 1032) = v9 - v10;
      return v11 + v10;
    }
    else
    {
      v12 = *(_DWORD *)(a1 + 1040);
      *(_DWORD *)(a1 + 1040) = 0;
      *(_DWORD *)(a1 + 1032) = v12;
      return 0;
    }
  }
  *(_DWORD *)(a1 + 1096) = sub_445F4(a1, 402);
  v13 = *(_WORD *)(a1 + 1044);
  v14 = *(_WORD *)(a1 + 1036);
  if ( v14 != 0 )
  {
    result = *(unsigned __int16 *)(a1 + 1044);
    if ( result < 0x3E7 )
    {
      *(_WORD *)(a1 + 1044) = v13 + 1;
      v15 = (unsigned __int16)(v14 - 1);
      *(_WORD *)(a1 + 1036) = v15;
      result = v15 << 16;
      v16 = *(unsigned __int16 *)(a1 + 1038);
      if ( v16 > (unsigned __int16)v15 )
      {
        v17 = *(unsigned __int16 *)(a1 + 1046);
        if ( v17 < 0x3E7 )
          *(_WORD *)(a1 + 1046) = v17 + 1;
        result = (unsigned __int16)(v16 - 1);
        *(_WORD *)(a1 + 1038) = result;
      }
    }
  }
  else
  {
    *(_WORD *)(a1 + 1044) = 0;
    *(_WORD *)(a1 + 1036) = v13;
    return 1036;
  }
  return result;
}

