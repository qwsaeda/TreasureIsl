/* ea:   0x00039CA4 */
/* name: sub_39CA4 */
/* size: 28 */
int __fastcall sub_39CA4(int result, int a2)
{
  *(_DWORD *)result = *(_DWORD *)a2;
  *(_BYTE *)(result + 4) = *(_BYTE *)(a2 + 4);
  return result;
}

