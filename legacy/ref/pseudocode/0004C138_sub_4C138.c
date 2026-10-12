/* ea:   0x0004C138 */
/* name: sub_4C138 */
/* size: 160 */
int __fastcall sub_4C138(int a1, int a2)
{
  int v4; // r5
  int result; // r0
  char v6; // [sp+0h] [bp-1Ch] BYREF

  v4 = a2 + 12;
  sub_4DD48(a2 + 12);
  *(_DWORD *)a2 = sub_16C9C((unsigned __int16 *)(a1 + 136), 2);
  *(_DWORD *)(a2 + 4) = *(unsigned __int16 *)(*(_DWORD *)(a1 + 20) + 26);
  *(_DWORD *)(a2 + 8) = *(_DWORD *)(*(_DWORD *)(a1 + 20) + 44);
  sub_4DFB4(v4, *(_DWORD *)(*(_DWORD *)(a1 + 20) + 28), *(_DWORD *)(*(_DWORD *)(a1 + 20) + 32), &v6);
  *(_WORD *)(a2 + 24) = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  result = *(_DWORD *)(a1 + 20);
  *(_BYTE *)(a2 + 26) = *(_BYTE *)(result + 49);
  return result;
}

