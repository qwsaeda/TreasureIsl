/* ea:   0x0004BF88 */
/* name: sub_4BF88 */
/* size: 164 */
int __fastcall sub_4BF88(_DWORD *a1, int a2)
{
  unsigned __int16 *v4; // r4
  int v5; // r0
  __int16 v6; // r0
  int result; // r0

  *(_DWORD *)a2 = 1;
  v4 = (unsigned __int16 *)(a1 + 34);
  *(_DWORD *)(a2 + 4) = a1[2];
  v5 = *(_DWORD *)(*(_DWORD *)(a1[6] + 32) + 4 * a1[2] + 80);
  *(_DWORD *)(a2 + 12) = 0;
  *(_DWORD *)(a2 + 8) = v5;
  *(_WORD *)(a2 + 16) = sub_16C9C(v4, 0);
  v6 = sub_16C9C(v4, 1);
  *(_DWORD *)(a2 + 20) = 0;
  *(_WORD *)(a2 + 18) = v6;
  *(_WORD *)(a2 + 24) = *(_WORD *)(a1[5] + 22);
  *(_WORD *)(a2 + 26) = *(_WORD *)(a1[5] + 24);
  result = *(unsigned __int16 *)(a1[5] + 26);
  *(_DWORD *)(a2 + 28) = result;
  return result;
}

