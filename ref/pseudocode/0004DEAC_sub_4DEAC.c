/* ea:   0x0004DEAC */
/* name: sub_4DEAC */
/* size: 88 */
_DWORD *__fastcall sub_4DEAC(int a1, _DWORD *a2, int a3)
{
  _DWORD *v3; // r5
  int v4; // r4
  int v5; // r3
  int v6; // r2

  v3 = a2;
  v4 = (*(_DWORD *)(a1 + 4) - a3) >> 3;
  if ( v4 > 0 )
  {
    v5 = a3 - (_DWORD)a2;
    do
    {
      --v4;
      v6 = *(_DWORD *)((char *)v3 + v5 + 4);
      *v3 = *(_DWORD *)((char *)v3 + v5);
      v3[1] = v6;
      v3 += 2;
    }
    while ( v4 != 0 );
  }
  *(_DWORD *)(a1 + 4) = v3;
  return a2;
}

