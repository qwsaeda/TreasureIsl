/* ea:   0x00017A90 */
/* name: sub_17A90 */
/* size: 264 */
int __fastcall sub_17A90(int a1, int a2)
{
  int v4; // r0
  unsigned int v5; // r2
  int v6; // r6
  int v7; // r3
  bool v8; // cf
  bool v9; // zf
  _DWORD *i; // r2
  _DWORD *v12; // r2

  v4 = sub_17B98();
  v5 = *(unsigned __int8 *)(a1 + 49);
  v6 = v5 - v4 + 1;
  v7 = v6 > 0;
  if ( v6 > 0 )
  {
    switch ( a2 )
    {
      case 1:
      case 2:
      case 3:
        v8 = true;
        v9 = v5 == 0;
        goto LABEL_7;
      case 4:
      case 5:
      case 6:
        v8 = v5 != 0;
        v9 = v5 == 1;
        goto LABEL_7;
      case 7:
      case 8:
        v8 = v5 >= 2;
        v9 = v5 == 2;
        goto LABEL_7;
      case 9:
        v8 = v5 >= 3;
        v9 = v5 == 3;
LABEL_7:
        v7 = !v9 && v8;
        break;
      default:
        break;
    }
  }
  if ( (_BYTE)v7 != 0 )
  {
    if ( a2 == 1 || a2 == 3 )
    {
      v12 = *(_DWORD **)(a1 + 28);
      v7 = 0;
      while ( v12 != *(_DWORD **)(a1 + 32) && v6 > 0 )
      {
        if ( *v12 == 0 )
          return 1;
        ++v12;
        --v6;
      }
    }
    else
    {
      for ( i = *(_DWORD **)(a1 + 28); i != *(_DWORD **)(a1 + 32); ++i )
      {
        if ( *i == a2 )
          return 0;
      }
    }
  }
  return v7;
}

