/* ea:   0x00053440 */
/* name: sub_53440 */
/* size: 76 */
int __fastcall sub_53440(int a1)
{
  *(_WORD *)a1 = 0;
  *(_WORD *)(a1 + 2) = 0;
  *(_WORD *)(a1 + 4) = 0;
  *(_WORD *)(a1 + 6) = 0;
  *(_DWORD *)(a1 + 8) = 0;
  *(_DWORD *)(a1 + 12) = 0;
  sub_32134(a1 + 16);
  sub_5348C(a1 + 28);
  *(_DWORD *)(a1 + 40) = 0;
  *(_WORD *)(a1 + 44) = 0;
  return a1;
}

