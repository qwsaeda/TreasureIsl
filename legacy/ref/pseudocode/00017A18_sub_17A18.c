/* ea:   0x00017A18 */
/* name: sub_17A18 */
/* size: 120 */
int __fastcall sub_17A18(int a1, _DWORD *a2)
{
  int v4; // r3
  int v5; // r1
  int result; // r0
  int v7; // r1

  v4 = *(_DWORD *)(a1 + 8);
  *(_DWORD *)(a1 + 8) = *(_DWORD *)(v4 + 12);
  v5 = *(_DWORD *)(v4 + 12);
  if ( v5 != 0 )
    *(_DWORD *)(v5 + 4) = a1;
  result = *(_DWORD *)(a1 + 4);
  *(_DWORD *)(v4 + 4) = result;
  if ( a1 == *a2 )
  {
    *a2 = v4;
    *(_DWORD *)(v4 + 12) = a1;
    *(_DWORD *)(a1 + 4) = v4;
  }
  else
  {
    v7 = *(_DWORD *)(a1 + 4);
    result = *(_DWORD *)(v7 + 12);
    if ( a1 == result )
      *(_DWORD *)(v7 + 12) = v4;
    else
      *(_DWORD *)(v7 + 8) = v4;
    *(_DWORD *)(v4 + 12) = a1;
    *(_DWORD *)(a1 + 4) = v4;
  }
  return result;
}

