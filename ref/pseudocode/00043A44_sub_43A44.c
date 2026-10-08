/* ea:   0x00043A44 */
/* name: sub_43A44 */
/* size: 216 */
int __fastcall sub_43A44(int result, int a2, int a3)
{
  _DWORD *v3; // r3
  int v4; // r2
  int v5; // r2
  int v6; // r1

  v3 = (_DWORD *)result;
  if ( a2 != 0 )
    return result;
  if ( a3 <= 38 )
  {
    if ( a3 != 38 )
    {
      if ( a3 == 13 )
      {
        result = *(_DWORD *)(result + 1032);
        if ( result == 4 )
        {
          v3[252] = 1;
          return 1;
        }
      }
      else if ( a3 == 37 )
      {
        result = *(_DWORD *)(result + 1032);
        if ( result != 4 )
        {
          v4 = result - 1;
          result = v3[257];
          v3[258] = v4;
          v3[257] = result - 1;
          if ( v4 == -1 )
          {
            v3[257] = 3;
            v3[258] = 3;
            return 3;
          }
        }
      }
      return result;
    }
LABEL_13:
    if ( *(_DWORD *)(result + 1032) == 4 )
    {
      result = *(_DWORD *)(result + 1028);
      v3[258] = result;
    }
    else
    {
      *(_DWORD *)(result + 1032) = 4;
      return 4;
    }
    return result;
  }
  if ( a3 != 39 )
  {
    if ( a3 != 40 )
      return result;
    goto LABEL_13;
  }
  result = *(_DWORD *)(result + 1032);
  if ( result != 4 )
  {
    v5 = result + 1;
    result = v3[257];
    v3[258] = v5;
    v6 = result + 1;
    if ( v5 == 4 )
      result = 0;
    v3[257] = v6;
    if ( v5 == 4 )
    {
      v3[257] = result;
      v3[258] = result;
    }
  }
  return result;
}

