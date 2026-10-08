/* ea:   0x0001C818 */
/* name: sub_1C818 */
/* size: 424 */
int __fastcall sub_1C818(int result, int a2, int a3)
{
  int v3; // r3
  int v4; // r0
  int v5; // r1
  int v6; // r0

  v3 = result;
  if ( a2 == 1 )
  {
    result = *(_DWORD *)(result + 1096);
    if ( result != 0 )
    {
      *(_DWORD *)(result + 20) = 1;
      *(_DWORD *)(v3 + 1096) = 0;
    }
    *(_DWORD *)(v3 + 1104) = 0;
    *(_DWORD *)(v3 + 1108) = 1;
    return result;
  }
  if ( a2 != 0 )
    return result;
  if ( a3 > 38 )
  {
    if ( a3 == 39 )
    {
      v6 = *(_DWORD *)(result + 1052);
      if ( v6 >= 4 && v6 <= 5 )
      {
        *(_DWORD *)(v3 + 1052) = 5;
        return 5;
      }
      return sub_1B940(v3);
    }
    if ( a3 != 40 )
      return result;
    *(_DWORD *)(result + 1104) = 0;
    result = *(_DWORD *)(result + 1052);
    *(_DWORD *)(v3 + 1108) = 1;
    if ( result == 1 )
    {
      if ( *(_DWORD *)(v3 + 1020) != 0 )
      {
LABEL_32:
        *(_DWORD *)(v3 + 1052) = 3;
        return 3;
      }
      else
      {
LABEL_21:
        *(_DWORD *)(v3 + 1052) = 2;
        return 2;
      }
    }
    else
    {
      if ( result > 3 && result <= 5 )
      {
LABEL_28:
        *(_DWORD *)(v3 + 1052) = 1;
        return result;
      }
      *(_DWORD *)(v3 + 1052) = ++result;
      if ( result > 3 )
        goto LABEL_30;
    }
  }
  else
  {
    switch ( a3 )
    {
      case 38:
        *(_DWORD *)(result + 1104) = 0;
        result = *(_DWORD *)(result + 1052);
        *(_DWORD *)(v3 + 1108) = 1;
        v5 = result - 1;
        if ( (unsigned int)(result - 1) <= 4 )
        {
          result = *((unsigned __int8 *)&jpt_1C8F0 + v5);
          switch ( v5 )
          {
            case 0:
              goto LABEL_30;
            case 1:
              break;
            case 2:
              result = *(_DWORD *)(v3 + 1020);
              if ( result == 0 )
                goto LABEL_21;
              goto LABEL_28;
            case 3:
            case 4:
              goto LABEL_32;
            default:
              abort();
          }
        }
        *(_DWORD *)(v3 + 1052) = v5;
        break;
      case 13:
        result = *(_DWORD *)(result + 1052);
        *(_DWORD *)(v3 + 1108) = 1;
        *(_DWORD *)(v3 + 1104) = 0;
        if ( result == 4 )
        {
          *(_DWORD *)(v3 + 1008) = 1;
          *(_BYTE *)(v3 + 1100) = 1;
        }
        if ( result == 5 )
        {
          *(_DWORD *)(v3 + 1008) = 1;
          *(_BYTE *)(v3 + 1100) = 0;
        }
        break;
      case 37:
        v4 = *(_DWORD *)(result + 1052);
        if ( v4 < 4 || v4 > 5 )
          return sub_1B940(v3);
LABEL_30:
        *(_DWORD *)(v3 + 1052) = 4;
        return 4;
      default:
        return result;
    }
  }
  return result;
}

