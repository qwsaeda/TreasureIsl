/* ea:   0x0004C7E0 */
/* name: sub_4C7E0 */
/* size: 188 */
int __fastcall sub_4C7E0(int a1, _DWORD *a2)
{
  int v4; // r1
  unsigned int v5; // r2
  unsigned int v6; // r1
  unsigned int v7; // r1
  unsigned int v8; // r0
  _DWORD *v9; // r2
  int *v10; // r5
  int i; // r4
  int result; // r0

  v4 = *a2 - sub_16C9C((unsigned __int16 *)(a1 + 136), 2);
  if ( v4 <= 0 )
  {
    v7 = abs32(v4);
    v9 = (_DWORD *)(a1 + 144);
    v8 = *(_DWORD *)(a1 + 144);
    if ( v7 < v8 )
      *v9 = v8 - v7;
    else
      *v9 = 0;
  }
  else
  {
    v5 = *(_DWORD *)(a1 + 148);
    v6 = *(_DWORD *)(a1 + 144) + v4;
    *(_DWORD *)(a1 + 144) = v6;
    if ( v6 > v5 )
      *(_DWORD *)(a1 + 144) = v5;
  }
  v10 = (int *)(a1 + 152);
  sub_16ED8((_DWORD *)(a1 + 152));
  sub_16F5C(a1 + 152);
  for ( i = a2[1]; ; i += 8 )
  {
    result = a2[2];
    if ( i == result )
      break;
    if ( *(_WORD *)(i + 4) != 0 )
      sub_30810(v10, *(unsigned __int8 *)(i + 6));
  }
  return result;
}

