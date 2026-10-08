/* ea:   0x00018350 */
/* name: sub_18350 */
/* size: 28 */
int __fastcall sub_18350(int result, int a2)
{
  *(_DWORD *)result = *(_DWORD *)a2;
  *(_BYTE *)(result + 4) = *(_BYTE *)(a2 + 4);
  return result;
}

