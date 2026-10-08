/* ea:   0x0004C0D0 */
/* name: sub_4C0D0 */
/* size: 104 */
int __fastcall sub_4C0D0(_DWORD *a1, int a2)
{
  int v4; // r0
  int result; // r0
  __int16 v6; // r1

  *(_DWORD *)a2 = 2;
  *(_DWORD *)(a2 + 4) = a1[2];
  v4 = *(_DWORD *)(*(_DWORD *)(a1[6] + 32) + 4 * a1[2] + 80);
  *(_DWORD *)(a2 + 12) = 0;
  *(_DWORD *)(a2 + 8) = v4;
  *(_WORD *)(a2 + 16) = *(_WORD *)(a1[4] + 12);
  result = a1[4];
  v6 = *(_WORD *)(result + 14);
  *(_DWORD *)(a2 + 20) = 0;
  *(_WORD *)(a2 + 18) = v6;
  *(_WORD *)(a2 + 24) = 0;
  *(_WORD *)(a2 + 26) = 0;
  *(_DWORD *)(a2 + 28) = 0;
  return result;
}

