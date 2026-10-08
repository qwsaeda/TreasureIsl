/* ea:   0x00064384 */
/* name: sub_64384 */
/* size: 104 */
_DWORD *__fastcall sub_64384(int a1, _DWORD *a2, int a3)
{
  _DWORD *v4; // r6
  int v5; // r5
  int v6; // r4
  int v7; // r2
  int v8; // r3
  int v9; // r0

  v4 = a2;
  v5 = (*(_DWORD *)(a1 + 4) - a3) >> 4;
  if ( v5 > 0 )
  {
    v6 = a3 - (_DWORD)a2;
    do
    {
      --v5;
      v7 = *(_DWORD *)((char *)v4 + v6 + 4);
      v8 = *(_DWORD *)((char *)v4 + v6 + 8);
      v9 = *(_DWORD *)((char *)v4 + v6 + 12);
      *v4 = *(_DWORD *)((char *)v4 + v6);
      v4[1] = v7;
      v4[2] = v8;
      v4[3] = v9;
      v4 += 4;
    }
    while ( v5 != 0 );
  }
  *(_DWORD *)(a1 + 4) = v4;
  return a2;
}

