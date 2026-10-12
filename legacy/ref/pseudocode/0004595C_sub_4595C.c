/* ea:   0x0004595C */
/* name: sub_4595C */
/* size: 140 */
int __fastcall sub_4595C(int a1)
{
  int v2; // r6
  int v3; // r0
  int v4; // r0

  v2 = a1 + 4;
  sub_1333C((_DWORD *)(a1 + 4));
  sub_1333C((_DWORD *)(v2 + 12));
  *(_DWORD *)a1 = &off_6FDC8;
  *(_BYTE *)(a1 + 40) = 0;
  v3 = sub_12F04(aGslucencyapp);
  sub_1318C((void **)(a1 + 16), (char *)aGslucencyapp, (int)&aGslucencyapp[v3]);
  v4 = sub_12F04(aLucencyApplica);
  sub_1318C((void **)v2, (char *)aLucencyApplica, (int)&aLucencyApplica[v4]);
  *(_DWORD *)(a1 + 28) = 0;
  *(_DWORD *)(a1 + 32) = 0;
  *(_BYTE *)(a1 + 36) = 1;
  return a1;
}

