/* ea:   0x0004C02C */
/* name: sub_4C02C */
/* size: 164 */
int __fastcall sub_4C02C(int a1, int a2)
{
  unsigned __int16 *v4; // r4
  int result; // r0
  __int16 v6; // r1

  *(_DWORD *)a2 = 0;
  v4 = (unsigned __int16 *)(a1 + 136);
  *(_DWORD *)(a2 + 4) = *(_DWORD *)(a1 + 8);
  *(_DWORD *)(a2 + 8) = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 32) + 4 * *(_DWORD *)(a1 + 8) + 80);
  *(_DWORD *)(a2 + 12) = sub_16C9C((unsigned __int16 *)(a1 + 136), 2);
  *(_WORD *)(a2 + 16) = sub_16C9C(v4, 0);
  *(_WORD *)(a2 + 18) = sub_16C9C(v4, 1);
  *(_DWORD *)(a2 + 20) = *(_DWORD *)(*(_DWORD *)(a1 + 16) + 8);
  *(_WORD *)(a2 + 24) = *(_WORD *)(*(_DWORD *)(a1 + 16) + 12);
  result = *(_DWORD *)(a1 + 16);
  v6 = *(_WORD *)(result + 14);
  *(_DWORD *)(a2 + 28) = 0;
  *(_WORD *)(a2 + 26) = v6;
  return result;
}

