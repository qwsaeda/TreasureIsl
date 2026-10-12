/* ea:   0x000631B0 */
/* name: sub_631B0 */
/* size: 116 */
int __fastcall sub_631B0(int a1, _WORD *a2)
{
  int v4; // r1
  int v5; // r0
  int v6; // r4
  void *v7; // r0
  int v9; // r1
  int v10; // r0

  v4 = *(_DWORD *)(a1 + 4);
  v5 = *(_DWORD *)(a1 + 8);
  if ( v4 == v5 )
  {
    v6 = 1;
    if ( v5 != 0 )
      v6 = 2 * v4;
    v7 = realloc(*(void **)a1, 2 * v6);
    if ( v7 == nullptr )
      return 0;
    *(_DWORD *)(a1 + 8) = v6;
    *(_DWORD *)a1 = v7;
  }
  v9 = *(_DWORD *)(a1 + 4) + 1;
  v10 = *(_DWORD *)a1 + 2 * v9;
  *(_DWORD *)(a1 + 4) = v9;
  if ( v10 != 2 )
    *(_WORD *)(v10 - 2) = *a2;
  return 1;
}

