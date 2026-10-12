/* ea:   0x00054054 */
/* name: sub_54054 */
/* size: 96 */
int __fastcall sub_54054(int a1, _DWORD *a2)
{
  _DWORD *v4; // r1
  int result; // r0

  v4 = *(_DWORD **)(a1 + 4);
  if ( v4 == *(_DWORD **)(a1 + 8) )
    return sub_54114(a1);
  if ( v4 != nullptr )
    *v4 = *a2;
  result = *(_DWORD *)(a1 + 4);
  *(_DWORD *)(a1 + 4) = result + 4;
  return result;
}

