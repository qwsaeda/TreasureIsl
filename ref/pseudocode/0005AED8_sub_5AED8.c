/* ea:   0x0005AED8 */
/* name: sub_5AED8 */
/* size: 16 */
int __fastcall sub_5AED8(int result, int a2)
{
  *(_DWORD *)(result + 28) = a2;
  *(_BYTE *)(result + 32) = 1;
  return result;
}

