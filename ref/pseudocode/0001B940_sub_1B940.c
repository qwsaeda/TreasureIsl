/* ea:   0x0001B940 */
/* name: sub_1B940 */
/* size: 108 */
int __fastcall sub_1B940(int a1)
{
  int v2; // r0
  int result; // r0

  v2 = *(_DWORD *)(a1 + 1020);
  if ( v2 != 0 )
  {
    if ( v2 == 1 )
    {
      sub_1C010(a1);
      result = *(_DWORD *)(a1 + 1096);
      *(_DWORD *)(result + 20) = 0;
      return result;
    }
    if ( v2 == 2 )
    {
      sub_1C524(a1);
      result = *(_DWORD *)(a1 + 1096);
      *(_DWORD *)(result + 20) = 0;
      return result;
    }
  }
  else
  {
    sub_1B9AC(a1);
  }
  result = *(_DWORD *)(a1 + 1096);
  *(_DWORD *)(result + 20) = 0;
  return result;
}

