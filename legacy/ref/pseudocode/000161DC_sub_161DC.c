/* ea:   0x000161DC */
/* name: sub_161DC */
/* size: 96 */
int __fastcall sub_161DC(int a1, _BYTE *a2)
{
  _BYTE *v4; // r1
  int result; // r0

  v4 = *(_BYTE **)(a1 + 4);
  if ( v4 == *(_BYTE **)(a1 + 8) )
    return sub_1631C(a1);
  if ( v4 != nullptr )
    *v4 = *a2;
  result = *(_DWORD *)(a1 + 4);
  *(_DWORD *)(a1 + 4) = result + 1;
  return result;
}

