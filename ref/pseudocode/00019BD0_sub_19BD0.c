/* ea:   0x00019BD0 */
/* name: sub_19BD0 */
/* size: 112 */
int __fastcall sub_19BD0(int a1, _DWORD *a2)
{
  int v4; // r1
  _DWORD *v5; // r2
  int v6; // r1
  int result; // r0

  v4 = *(_DWORD *)(a1 + 4);
  if ( v4 == *(_DWORD *)(a1 + 8) )
    return sub_19C40(a1);
  v5 = *(_DWORD **)(a1 + 4);
  if ( v4 != 0 )
  {
    v6 = a2[1];
    *v5 = *a2;
    v5[1] = v6;
  }
  result = *(_DWORD *)(a1 + 4);
  *(_DWORD *)(a1 + 4) = result + 8;
  return result;
}

