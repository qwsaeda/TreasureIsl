/* ea:   0x00060BB8 */
/* name: sub_60BB8 */
/* size: 156 */
int __fastcall sub_60BB8(int a1, int a2, int a3)
{
  int v6; // r8
  int v7; // r6
  int i; // r5
  int v9; // r4

  v6 = ((int (*)(void))sub_61598)();
  *(_DWORD *)(v6 + 4) = a3;
  if ( *(_DWORD *)(a2 + 12) != 0 )
    *(_DWORD *)(v6 + 12) = sub_60BB8(a1, *(_DWORD *)(a2 + 12), v6);
  v7 = v6;
  for ( i = *(_DWORD *)(a2 + 8); i != 0; i = *(_DWORD *)(i + 8) )
  {
    v9 = sub_61598(a1, i);
    *(_DWORD *)(v7 + 8) = v9;
    *(_DWORD *)(v9 + 4) = v7;
    if ( *(_DWORD *)(i + 12) != 0 )
      *(_DWORD *)(v9 + 12) = sub_60BB8(a1, *(_DWORD *)(i + 12), v9);
    v7 = v9;
  }
  return v6;
}

