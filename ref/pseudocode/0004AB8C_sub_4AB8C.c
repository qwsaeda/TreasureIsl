/* ea:   0x0004AB8C */
/* name: sub_4AB8C */
/* size: 140 */
int __fastcall sub_4AB8C(_DWORD *a1)
{
  int v1; // r2
  int v2; // r1
  int result; // r0
  int v4; // r1

  v1 = 0;
  switch ( a1[31] )
  {
    case 0:
    case 1:
      v4 = a1[10];
      if ( v4 == 0 )
        __debugbreak();
      v1 = *(_DWORD *)(a1[3] + 16) - *(_DWORD *)(v4 + 8) - a1[30] + 33;
      goto LABEL_8;
    case 2:
    case 3:
      v2 = a1[10];
      if ( v2 == 0 )
        __debugbreak();
      result = *(_DWORD *)(a1[3] + 16) - *(_DWORD *)(v2 + 8) + a1[30] + 33;
      break;
    default:
LABEL_8:
      result = v1;
      break;
  }
  return result;
}

