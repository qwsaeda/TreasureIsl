/* ea:   0x000194F0 */
/* name: sub_194F0 */
/* size: 16 */
int __fastcall sub_194F0(int result, int a2)
{
  *(_DWORD *)(result + 16) = a2;
  *(_BYTE *)(result + 12) = 1;
  return result;
}

