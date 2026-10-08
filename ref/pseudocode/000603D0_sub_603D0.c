/* ea:   0x000603D0 */
/* name: sub_603D0 */
/* size: 28 */
int __fastcall sub_603D0(int result, int a2)
{
  *(_DWORD *)result = *(_DWORD *)a2;
  *(_BYTE *)(result + 4) = *(_BYTE *)(a2 + 4);
  return result;
}

