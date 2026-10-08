/* ea:   0x0004EB90 */
/* name: sub_4EB90 */
/* size: 172 */
int __fastcall sub_4EB90(int result, int a2, int a3)
{
  _DWORD *v3; // r3
  int v4; // r2

  v3 = (_DWORD *)result;
  if ( a2 == 0 )
  {
    switch ( a3 )
    {
      case 13:
        v4 = *(_DWORD *)(result + 1020);
        if ( v4 == 5 )
        {
          *(_DWORD *)(result + 1028) = 5;
          *(_DWORD *)(result + 1008) = 1;
          return 5;
        }
        else if ( *(_DWORD *)(result + 1024) != 0 || (result += 16 * v4, *(_DWORD *)(result + 1032) != 0) )
        {
          v3[257] = v4;
          v3[252] = 1;
          return 1;
        }
        break;
      case 38:
        result = *(_DWORD *)(result + 1020);
        v3[255] = result - 1;
        if ( result - 1 < 0 )
        {
          v3[255] = 5;
          return 5;
        }
        break;
      case 40:
        result = *(_DWORD *)(result + 1020);
        v3[255] = result + 1;
        if ( result + 1 > 5 )
        {
          v3[255] = 0;
          return 0;
        }
        break;
      default:
        break;
    }
  }
  return result;
}

