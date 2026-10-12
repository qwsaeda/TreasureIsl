/* ea:   0x00065E88 */
/* name: sub_65E88 */
/* size: 28 */
int __fastcall sub_65E88(int result, int a2)
{
  *(_DWORD *)result = *(_DWORD *)a2;
  *(_BYTE *)(result + 4) = *(_BYTE *)(a2 + 4);
  return result;
}

