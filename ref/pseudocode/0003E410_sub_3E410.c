/* ea:   0x0003E410 */
/* name: sub_3E410 */
/* size: 28 */
int __fastcall sub_3E410(int result, int a2)
{
  *(_DWORD *)result = *(_DWORD *)a2;
  *(_BYTE *)(result + 4) = *(_BYTE *)(a2 + 4);
  return result;
}

