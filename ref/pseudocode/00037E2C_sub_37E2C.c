/* ea:   0x00037E2C */
/* name: sub_37E2C */
/* size: 148 */
_DWORD *__fastcall sub_37E2C(_DWORD *a1, int a2, int a3)
{
  _BYTE v7[6]; // [sp+2h] [bp-26h] BYREF
  void *v8[8]; // [sp+8h] [bp-20h] BYREF

  sub_380E8(v8, v7[0], ((*(_DWORD *)(a2 + 4) - *(_DWORD *)a2) >> 1) + ((*(_DWORD *)(a3 + 4) - *(_DWORD *)a3) >> 1), v7);
  sub_18548((int *)v8, *(_WORD **)a2, *(_DWORD *)(a2 + 4));
  sub_18548((int *)v8, *(_WORD **)a3, *(_DWORD *)(a3 + 4));
  sub_132FC(a1, v8);
  sub_12F48(v8);
  return a1;
}

