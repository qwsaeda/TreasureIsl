/* ea:   0x000540B4 */
/* name: sub_540B4 */
/* size: 96 */
int __fastcall sub_540B4(int a1, int a2)
{
  int v4; // r1
  int result; // r0

  v4 = *(_DWORD *)(a1 + 4);
  if ( v4 == *(_DWORD *)(a1 + 8) )
    return sub_5423C(a1);
  if ( v4 != 0 )
    sub_5454C(v4, a2);
  result = *(_DWORD *)(a1 + 4);
  *(_DWORD *)(a1 + 4) = result + 36;
  return result;
}

