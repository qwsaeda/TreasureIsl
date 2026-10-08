/* ea:   0x000603B4 */
/* name: sub_603B4 */
/* size: 28 */
int __fastcall sub_603B4(int result, int a2)
{
  *(_DWORD *)result = *(_DWORD *)a2;
  *(_BYTE *)(result + 4) = *(_BYTE *)(a2 + 4);
  return result;
}

