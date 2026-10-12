/* ea:   0x000375A8 */
/* name: sub_375A8 */
/* size: 28 */
int __fastcall sub_375A8(int result, int a2)
{
  *(_DWORD *)result = *(_DWORD *)a2;
  *(_BYTE *)(result + 4) = *(_BYTE *)(a2 + 4);
  return result;
}

