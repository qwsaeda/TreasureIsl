/* ea:   0x0003FDF4 */
/* name: sub_3FDF4 */
/* size: 140 */
int __fastcall sub_3FDF4(int result, int a2, int a3)
{
  int v3; // r3
  int v4; // r2
  int v5; // r2

  v3 = result;
  if ( a2 == 0 )
  {
    if ( a3 > 38 )
    {
      if ( a3 == 40 )
      {
        v5 = *(_DWORD *)(result + 1024);
        result = *(_DWORD *)(result + 1028);
        if ( v5 < result )
        {
          result = v3 + 4 * v5;
          if ( *(_DWORD *)(result + 2060) != 0 )
          {
            *(_DWORD *)(v3 + 1024) = v5 + 1;
            return v5 + 1;
          }
        }
      }
    }
    else if ( a3 == 38 )
    {
      v4 = *(_DWORD *)(result + 1024);
      if ( v4 > 0 )
      {
        result += 4 * v4;
        if ( *(_DWORD *)(result + 2052) != 0 )
        {
          *(_DWORD *)(v3 + 1024) = v4 - 1;
          return v4 - 1;
        }
      }
    }
    else if ( a3 == 13 || a3 == 27 )
    {
      *(_DWORD *)(result + 1008) = 1;
      return 1;
    }
  }
  return result;
}

