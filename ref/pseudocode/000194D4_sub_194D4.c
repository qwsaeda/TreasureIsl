/* ea:   0x000194D4 */
/* name: sub_194D4 */
/* size: 16 */
int __fastcall sub_194D4(int result, int a2)
{
  *(_DWORD *)(result + 8) = a2;
  *(_BYTE *)(result + 4) = 1;
  return result;
}

