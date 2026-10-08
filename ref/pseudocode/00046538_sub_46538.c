/* ea:   0x00046538 */
/* name: sub_46538 */
/* size: 220 */
int __fastcall sub_46538(int result, int a2, int a3)
{
  _DWORD *v3; // r5
  int v4; // r0
  bool v5; // zf

  v3 = (_DWORD *)result;
  if ( a2 == 0 )
  {
    result = *(_DWORD *)(result + 1024);
    if ( result != 0 )
    {
      if ( a3 == 13 )
      {
        result = v3[257];
        if ( result == 1 )
        {
          v3[252] = 1;
          v3[256] = 1;
        }
        else if ( result == 2 )
        {
          v3[252] = 1;
          v3[256] = 0;
          return 1;
        }
      }
      else if ( a3 == 37 || a3 == 39 )
      {
        v4 = v3[258];
        v5 = v4 == 0;
        if ( v4 != 0 )
          *(_DWORD *)(v4 + 20) = 1;
        result = v3[257];
        if ( !v5 )
          v3[258] = 0;
        if ( result == 1 )
        {
          result = (int)sub_445F4((int)v3, 22);
          v3[258] = result;
          *(_DWORD *)(result + 20) = 0;
          v3[257] = 2;
        }
        else if ( result == 2 )
        {
          result = (int)sub_445F4((int)v3, 12);
          v3[258] = result;
          *(_DWORD *)(result + 20) = 0;
          v3[257] = 1;
        }
      }
    }
    else
    {
      v3[252] = 1;
      return 1;
    }
  }
  return result;
}

