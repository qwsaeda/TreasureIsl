/* ea:   0x0005AE80 */
/* name: sub_5AE80 */
/* size: 44 */
int __fastcall sub_5AE80(int result)
{
  *(_DWORD *)result = off_7BB60;
  *(_BYTE *)(result + 22) = 1;
  *(_BYTE *)(result + 32) = 1;
  *(_BYTE *)(result + 21) = 0;
  return result;
}

