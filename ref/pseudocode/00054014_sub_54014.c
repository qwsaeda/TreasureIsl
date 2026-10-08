/* ea:   0x00054014 */
/* name: sub_54014 */
/* size: 64 */
int __fastcall sub_54014(_DWORD *a1, unsigned __int8 a2, char a3, int a4)
{
  int result; // r0

  *(_BYTE *)(a1[8] + 12 * a2 - 4) = a3;
  *(_DWORD *)(a1[8] + 12 * a2 - 12) = a4;
  result = a1[8];
  *(_DWORD *)(result + 12 * a2 - 8) = *(_DWORD *)(*a1 + 148);
  return result;
}

