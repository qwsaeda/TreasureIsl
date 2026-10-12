/* ea:   0x00058808 */
/* name: sub_58808 */
/* size: 220 */
int __fastcall sub_58808(int a1, unsigned __int8 *a2)
{
  unsigned __int8 *v3; // r5
  int v4; // r0
  int v5; // r1
  int v6; // r2
  int v7; // r3
  __int16 v8; // t1
  unsigned __int8 *v9; // r5
  __int16 v10; // t1
  int v12; // r4
  int v13; // r3
  __int16 v14; // r1
  unsigned __int8 *v15; // r5
  __int16 v16; // t1
  __int16 v17; // r1
  __int16 v18; // t1

  v3 = a2;
  v4 = *(_DWORD *)a2;
  v5 = *((_DWORD *)a2 + 1);
  v6 = *((_DWORD *)v3 + 2);
  v7 = *((_DWORD *)v3 + 3);
  v3 += 18;
  *(_DWORD *)(a1 + 40) = v4;
  *(_DWORD *)(a1 + 44) = v5;
  *(_DWORD *)(a1 + 48) = v6;
  *(_DWORD *)(a1 + 52) = v7;
  LOWORD(v5) = v3[1];
  v8 = *v3;
  v3 += 2;
  *(_WORD *)(a1 + 56) = v8 | ((_WORD)v5 << 8);
  LOWORD(v5) = v3[1];
  v10 = *v3;
  v9 = v3 + 2;
  *(_WORD *)(a1 + 58) = v10 | ((_WORD)v5 << 8);
  if ( (unsigned __int16)(v10 | ((_WORD)v5 << 8)) != 7 )
    return -1;
  v12 = 0;
  v13 = a1;
  do
  {
    v14 = v9[1];
    ++v12;
    v16 = *v9;
    v15 = v9 + 2;
    *(_WORD *)(v13 + 60) = v16 | (v14 << 8);
    v17 = v15[1];
    v18 = *v15;
    v9 = v15 + 2;
    *(_WORD *)(v13 + 62) = v18 | (v17 << 8);
    v13 += 4;
  }
  while ( v12 < *(unsigned __int16 *)(a1 + 58) );
  return 0;
}

