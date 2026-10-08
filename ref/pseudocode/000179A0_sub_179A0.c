/* ea:   0x000179A0 */
/* name: sub_179A0 */
/* size: 120 */
int __fastcall sub_179A0(int a1, _DWORD *a2)
{
  int v4; // r3
  int v5; // r1
  int result; // r0
  int v7; // r1

  v4 = *(_DWORD *)(a1 + 12);
  *(_DWORD *)(a1 + 12) = *(_DWORD *)(v4 + 8);
  v5 = *(_DWORD *)(v4 + 8);
  if ( v5 != 0 )
    *(_DWORD *)(v5 + 4) = a1;
  result = *(_DWORD *)(a1 + 4);
  *(_DWORD *)(v4 + 4) = result;
  if ( a1 == *a2 )
  {
    *a2 = v4;
    *(_DWORD *)(v4 + 8) = a1;
    *(_DWORD *)(a1 + 4) = v4;
  }
  else
  {
    v7 = *(_DWORD *)(a1 + 4);
    result = *(_DWORD *)(v7 + 8);
    if ( a1 == result )
      *(_DWORD *)(v7 + 8) = v4;
    else
      *(_DWORD *)(v7 + 12) = v4;
    *(_DWORD *)(v4 + 8) = a1;
    *(_DWORD *)(a1 + 4) = v4;
  }
  return result;
}

