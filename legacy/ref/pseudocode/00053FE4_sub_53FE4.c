/* ea:   0x00053FE4 */
/* name: sub_53FE4 */
/* size: 48 */
int __fastcall sub_53FE4(int *a1, unsigned __int8 a2, int a3)
{
  int v3; // r3
  int result; // r0

  *(_DWORD *)(a1[1] + 8 * a2 - 8) = a3;
  v3 = *a1;
  result = a1[1];
  *(_DWORD *)(result + 8 * a2 - 4) = *(_DWORD *)(v3 + 148);
  return result;
}

