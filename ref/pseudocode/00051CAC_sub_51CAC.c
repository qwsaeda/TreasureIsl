/* ea:   0x00051CAC */
/* name: sub_51CAC */
/* size: 220 */
int __fastcall sub_51CAC(int a1, int a2)
{
  unsigned __int16 *v4; // r4
  int result; // r0

  v4 = (unsigned __int16 *)(a1 + 136);
  *(_DWORD *)a2 = *(_DWORD *)(a1 + 8);
  *(_DWORD *)(a2 + 4) = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 32) + 4 * *(_DWORD *)(a1 + 8) + 80);
  *(_DWORD *)(a2 + 8) = sub_16C9C((unsigned __int16 *)(a1 + 136), 2);
  *(_DWORD *)(a2 + 12) = *(_DWORD *)(*(_DWORD *)(a1 + 16) + 8);
  *(_WORD *)(a2 + 16) = sub_16C9C(v4, 0);
  *(_WORD *)(a2 + 18) = *(_WORD *)(*(_DWORD *)(a1 + 16) + 12);
  *(_WORD *)(a2 + 20) = sub_16C9C(v4, 1);
  *(_WORD *)(a2 + 22) = *(_WORD *)(*(_DWORD *)(a1 + 16) + 14);
  *(_WORD *)(a2 + 24) = sub_16C9C(v4, 3);
  *(_BYTE *)(a2 + 27) = sub_16C9C(v4, 4);
  *(_WORD *)(a2 + 28) = *(_WORD *)(*(_DWORD *)(a1 + 16) + 16);
  result = *(_DWORD *)(a1 + 200);
  *(_BYTE *)(a2 + 26) = result;
  return result;
}

