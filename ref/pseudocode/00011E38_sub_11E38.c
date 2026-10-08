/* ea:   0x00011E38 */
/* name: sub_11E38 */
/* size: 1088 */
int __fastcall sub_11E38(int result, int a2, int a3)
{
  _DWORD *v3; // r5
  int v4; // r0
  int v5; // r0
  int v6; // r0
  int v7; // r1
  int v8; // r3
  int v9; // r2
  int v10; // r1
  int v11; // r0
  int v12; // r0
  int v13; // r0
  int v14; // r0
  int v15; // r0

  v3 = (_DWORD *)result;
  if ( a2 != 0 )
    return result;
  v4 = *(_DWORD *)(result + 1268);
  switch ( v4 )
  {
    case 30:
    case 40:
      return (*(int (__fastcall **)(_DWORD, _DWORD))(*(_DWORD *)v3[318] + 16))(v3[318], 0);
    case 10:
      result = v3[318];
      if ( result != 0 )
        result = (**(int (__fastcall ***)(_DWORD, int))v3[318])(v3[318], 1);
      v3[317] = 0;
      v3[318] = 0;
      return result;
    case 20:
      result = v3[318];
      if ( result != 0 )
        result = (**(int (__fastcall ***)(_DWORD, int))v3[318])(v3[318], 1);
      v3[317] = 0;
      v3[318] = 0;
      return result;
    default:
      break;
  }
  result = v3[312];
  if ( result == 10 )
  {
    if ( a3 > 38 )
    {
      if ( a3 == 39 )
      {
        v11 = v3[282];
        if ( v11 == 7 )
          goto LABEL_32;
        if ( v11 == 8 )
          goto LABEL_20;
        goto LABEL_44;
      }
      if ( a3 == 40 )
      {
        result = v3[282];
        v8 = v3[314];
        v9 = result + 1;
        v3[282] = result + 1;
        if ( v8 >= 5 )
        {
          if ( result == 5 )
          {
            v10 = v3[283] - 1;
            v3[283] = v10;
            if ( v10 == 4 - v8 )
              v3[283] = 5 - v8;
            v3[282] = 5;
            return 5;
          }
        }
        else
        {
          result = v3[283];
          if ( v9 > v8 + result )
          {
            v3[282] = v9 - 1;
            return v9 - 1;
          }
        }
      }
    }
    else
    {
      if ( a3 != 38 )
      {
        if ( a3 != 13 )
        {
          if ( a3 != 37 )
            return result;
          v5 = v3[282];
          if ( v5 != 7 )
          {
            if ( v5 != 8 )
            {
LABEL_20:
              v3[282] = 7;
              return 7;
            }
LABEL_32:
            v3[282] = 1;
            return 1;
          }
LABEL_44:
          v3[282] = 8;
          return 8;
        }
        v6 = v3[282];
        if ( v6 == 7 )
        {
          v3[312] = 20;
          v3[282] = 0;
          v3[315] = 1;
          result = sub_445F4(v3, 12);
          *(_DWORD *)(result + 20) = 1;
          v3[313] = 0;
          return result;
        }
        if ( v6 != 8 )
          return sub_12434(v3);
LABEL_24:
        v3[252] = 1;
        return 1;
      }
      result = v3[282];
      if ( result == 7 || result == 8 )
      {
        v3[282] = 1;
        return result;
      }
      v3[282] = --result;
      if ( result == 0 )
      {
        v7 = v3[283] + 1;
        v3[283] = v7;
        if ( v7 > 0 )
          v3[283] = 0;
        goto LABEL_32;
      }
    }
  }
  else if ( a3 > 38 )
  {
    if ( a3 == 39 )
    {
      result = v3[315];
      switch ( result )
      {
        case 1:
LABEL_67:
          v3[315] = 3;
          result = 3;
          break;
        case 2:
LABEL_51:
          v3[315] = 1;
          result = 1;
          break;
        case 3:
LABEL_58:
          v3[315] = 5;
          result = 5;
          break;
        case 4:
LABEL_50:
          v3[315] = 2;
          result = 2;
          break;
        case 5:
          v3[315] = 7;
          result = sub_445F4(v3, 22);
          v3[313] = result;
          *(_DWORD *)(result + 20) = 0;
          break;
        case 6:
          v14 = v3[313];
          if ( v14 != 0 )
            *(_DWORD *)(v14 + 20) = 1;
LABEL_71:
          v3[315] = 4;
          result = 4;
          break;
        case 7:
          v3[315] = 6;
          v15 = v3[313];
          if ( v15 != 0 )
            *(_DWORD *)(v15 + 20) = 1;
          result = sub_445F4(v3, 12);
          v3[313] = result;
          *(_DWORD *)(result + 20) = 0;
          break;
        default:
          return result;
      }
    }
    else if ( a3 == 40 )
    {
      v3[315] = 7;
      return 7;
    }
  }
  else
  {
    switch ( a3 )
    {
      case 38:
        v3[315] = 6;
        return 6;
      case 13:
        result = v3[315];
        if ( result == 6 )
          return result;
        if ( result != 7 )
          return sub_12278(v3);
        goto LABEL_24;
      case 37:
        result = v3[315];
        switch ( result )
        {
          case 1:
            goto LABEL_50;
          case 2:
            goto LABEL_71;
          case 3:
            goto LABEL_51;
          case 4:
            v3[315] = 6;
            result = sub_445F4(v3, 12);
            v3[313] = result;
            *(_DWORD *)(result + 20) = 0;
            break;
          case 5:
            goto LABEL_67;
          case 6:
            v3[315] = 7;
            v12 = v3[313];
            if ( v12 != 0 )
              *(_DWORD *)(v12 + 20) = 1;
            result = sub_445F4(v3, 22);
            v3[313] = result;
            *(_DWORD *)(result + 20) = 0;
            break;
          case 7:
            v13 = v3[313];
            if ( v13 != 0 )
              *(_DWORD *)(v13 + 20) = 1;
            goto LABEL_58;
          default:
            return result;
        }
        break;
      default:
        break;
    }
  }
  return result;
}

