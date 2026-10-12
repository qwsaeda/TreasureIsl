/* ea:   0x000537F8 */
/* name: sub_537F8 */
/* size: 16 */
int __fastcall sub_537F8(int a1, int a2, int a3, int a4)
{
  int result; // r0

  result = *(_DWORD *)(a1 + 8);
  *(_DWORD *)(*(_DWORD *)(result + 4 * a2) + 4 * a3) = a4;
  return result;
}

