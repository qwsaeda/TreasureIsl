/* ea:   0x00039C34 */
/* name: sub_39C34 */
/* size: 28 */
int __fastcall sub_39C34(int result, int a2)
{
  *(_DWORD *)result = *(_DWORD *)a2;
  *(_BYTE *)(result + 4) = *(_BYTE *)(a2 + 4);
  return result;
}

