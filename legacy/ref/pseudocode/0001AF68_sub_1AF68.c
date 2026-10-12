/* ea:   0x0001AF68 */
/* name: sub_1AF68 */
/* size: 164 */
_DWORD *__fastcall sub_1AF68(int a1, _DWORD *a2, int a3)
{
  _DWORD *v4; // r7
  int v5; // r6
  int v6; // r4
  int v7; // r1
  int v8; // r2
  int v9; // r3
  int v10; // r1
  int v11; // r2
  int v12; // r3

  v4 = a2;
  v5 = (*(_DWORD *)(a1 + 4) - a3) / 36;
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
      v10 = *(_DWORD *)((char *)v4 + v6 + 20);
      v11 = *(_DWORD *)((char *)v4 + v6 + 24);
      v12 = *(_DWORD *)((char *)v4 + v6 + 28);
      v4[4] = *(_DWORD *)((char *)v4 + v6 + 16);
      v4[5] = v10;
      v4[6] = v11;
      v4[7] = v12;
      v4[8] = *(_DWORD *)((char *)v4 + v6 + 32);
      v4 += 9;
    }
    while ( v5 != 0 );
  }
  *(_DWORD *)(a1 + 4) = v4;
  return a2;
}

