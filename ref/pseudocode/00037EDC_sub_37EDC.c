/* ea:   0x00037EDC */
/* name: sub_37EDC */
/* size: 148 */
_DWORD *__fastcall sub_37EDC(_DWORD *a1, int a2, char *a3)
{
  int v6; // r5
  _BYTE v8[6]; // [sp+2h] [bp-26h] BYREF
  void *v9[8]; // [sp+8h] [bp-20h] BYREF

  v6 = sub_12F04(a3);
  sub_380E8(v9, v8[0], v6 + ((*(_DWORD *)(a2 + 4) - *(_DWORD *)a2) >> 1), v8);
  sub_18548((int *)v9, *(_WORD **)a2, *(_DWORD *)(a2 + 4));
  sub_136F8(v9, a3, &a3[2 * v6]);
  sub_132FC(a1, v9);
  sub_12F48(v9);
  return a1;
}

