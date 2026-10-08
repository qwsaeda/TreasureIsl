/* ea:   0x00011AF8 */
/* name: sub_11AF8 */
/* size: 168 */
int __fastcall sub_11AF8(int a1, int a2)
{
  __int16 v3; // r0
  int *v4; // r5
  int *v5; // r6
  int v6; // r3

  *(_BYTE *)(a1 + 1050) = *(_BYTE *)(a2 + 26);
  *(_DWORD *)(a1 + 1024) = *(_DWORD *)a2;
  *(_DWORD *)(a1 + 1028) = *(_DWORD *)(a2 + 4);
  *(_DWORD *)(a1 + 1032) = *(_DWORD *)(a2 + 8);
  v3 = *(_WORD *)(a2 + 24);
  *(_DWORD *)(a1 + 1264) = 0;
  *(_WORD *)(a1 + 1048) = v3;
  v4 = *(int **)(a2 + 12);
  v5 = *(int **)(a2 + 16);
  while ( v4 != v5 )
  {
    sub_12EA4(a1 + 1036, v4);
    v6 = *v4++;
    if ( v6 != 0 )
      ++*(_DWORD *)(a1 + 1264);
  }
  sub_129D4(a1);
  *(_DWORD *)(a1 + 1132) = 0;
  *(_DWORD *)(a1 + 1248) = 10;
  *(_DWORD *)(a1 + 1128) = 1;
  return 1;
}

