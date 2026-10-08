/* ea:   0x00034950 */
/* name: sub_34950 */
/* size: 116 */
_DWORD *__fastcall sub_34950(int a1, _DWORD *a2, int a3)
{
  _DWORD *v3; // r6
  int v4; // r5
  int v5; // r4
  int v6; // r2
  int v7; // r3

  v3 = a2;
  v4 = (*(_DWORD *)(a1 + 4) - a3) / 12;
  if ( v4 > 0 )
  {
    v5 = a3 - (_DWORD)a2;
    do
    {
      --v4;
      v6 = *(_DWORD *)((char *)v3 + v5 + 4);
      v7 = *(_DWORD *)((char *)v3 + v5 + 8);
      *v3 = *(_DWORD *)((char *)v3 + v5);
      v3[1] = v6;
      v3[2] = v7;
      v3 += 3;
    }
    while ( v4 != 0 );
  }
  *(_DWORD *)(a1 + 4) = v3;
  return a2;
}

