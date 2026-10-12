/* ea:   0x00055210 */
/* name: sub_55210 */
/* size: 152 */
BOOL __fastcall sub_55210(int a1)
{
  int v2; // r0
  int v3; // r6
  int *v4; // r4
  int v5; // r5
  int v6; // r3
  int v7; // t1
  int v8; // r2

  v2 = *(_DWORD *)(a1 + 1064);
  *(_DWORD *)(a1 + 1132) = 0;
  v3 = 0;
  *(_DWORD *)(a1 + 1136) = 0;
  if ( v2 > 0 )
  {
    v4 = *(int **)(a1 + 1040);
    v5 = a1;
    do
    {
      ++v3;
      v6 = *(_DWORD *)(a1 + 1132);
      *(_DWORD *)(a1 + 1136) += *(_DWORD *)(v5 + 1112);
      v7 = *v4;
      v4 += 2;
      v8 = *(_DWORD *)(v5 + 1112);
      v5 += 4;
      *(_DWORD *)(a1 + 1132) = v6 + v8 * v7;
    }
    while ( v3 < *(_DWORD *)(a1 + 1064) );
  }
  return *(_DWORD *)(a1 + 1136) <= (int)*(unsigned __int16 *)(a1 + 1052)
      && *(_DWORD *)(a1 + 1132) <= *(_DWORD *)(a1 + 1024);
}

