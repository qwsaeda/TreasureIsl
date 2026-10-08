/* ea:   0x00065AB4 */
/* name: sub_65AB4 */
/* size: 96 */
int __fastcall sub_65AB4(int a1, _DWORD *a2)
{
  _DWORD *v4; // r1
  int result; // r0

  v4 = *(_DWORD **)(a1 + 4);
  if ( v4 == *(_DWORD **)(a1 + 8) )
    return sub_65BA8(a1);
  if ( v4 != nullptr )
    *v4 = *a2;
  result = *(_DWORD *)(a1 + 4);
  *(_DWORD *)(a1 + 4) = result + 4;
  return result;
}

