/* ea:   0x0003CC18 */
/* name: sub_3CC18 */
/* size: 96 */
int __fastcall sub_3CC18(int a1, _DWORD *a2)
{
  _DWORD *v4; // r1
  int result; // r0

  v4 = *(_DWORD **)(a1 + 4);
  if ( v4 == *(_DWORD **)(a1 + 8) )
    return sub_3CC78(a1);
  if ( v4 != nullptr )
    *v4 = *a2;
  result = *(_DWORD *)(a1 + 4);
  *(_DWORD *)(a1 + 4) = result + 4;
  return result;
}

