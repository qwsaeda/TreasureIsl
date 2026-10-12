/* ea:   0x0001DCEC */
/* name: sub_1DCEC */
/* size: 36 */
int __fastcall sub_1DCEC(int result)
{
  *(_DWORD *)result = &off_6C910;
  *(_BYTE *)(result + 12) = 0;
  *(_BYTE *)(result + 4) = 0;
  return result;
}

