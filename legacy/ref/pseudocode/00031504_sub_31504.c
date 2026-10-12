/* ea:   0x00031504 */
/* name: sub_31504 */
/* size: 20 */
int __fastcall sub_31504(int a1)
{
  int result; // r0

  *(_DWORD *)(a1 + 176) = 6;
  result = a1 + 176;
  *(_WORD *)(result + 4) = 3;
  return result;
}

