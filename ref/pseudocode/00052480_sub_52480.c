/* ea:   0x00052480 */
/* name: sub_52480 */
/* size: 148 */
unsigned int __fastcall sub_52480(int a1, unsigned int a2)
{
  unsigned int v4; // r3
  unsigned int v5; // r2
  unsigned int v6; // r6
  unsigned int v7; // r0
  _DWORD *v8; // r1
  unsigned int result; // r0
  int v10; // r1
  unsigned int v11; // r0
  int v12; // r2

  v4 = sub_16C9C((unsigned __int16 *)(a1 + 136), 2);
  v5 = *(_DWORD *)(*(_DWORD *)(a1 + 16) + 8);
  v6 = 0;
  v8 = (_DWORD *)(a1 + 144);
  v7 = *(_DWORD *)(a1 + 144);
  if ( a2 >= v7 )
    *v8 = 0;
  else
    *v8 = v7 - a2;
  result = v5 + v4;
  if ( a2 <= v5 + v4 )
  {
    if ( a2 > v4 )
    {
      v12 = *(_DWORD *)(a1 + 16);
      result = *(_DWORD *)(v12 + 8);
      if ( a2 - v4 <= result )
        v6 = result - (a2 - v4);
      *(_DWORD *)(v12 + 8) = v6;
    }
  }
  else
  {
    v10 = *(_DWORD *)(a1 + 16);
    v11 = *(_DWORD *)(v10 + 8);
    if ( v5 <= v11 )
      v6 = v11 - v5;
    *(_DWORD *)(v10 + 8) = v6;
    *(_DWORD *)(a1 + 132) = a2 - v5 - v4;
    return a2 - v5;
  }
  return result;
}

