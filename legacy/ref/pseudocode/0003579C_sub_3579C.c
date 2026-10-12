/* ea:   0x0003579C */
/* name: sub_3579C */
/* size: 28 */
int __fastcall sub_3579C(int result, int a2)
{
  *(_DWORD *)result = *(_DWORD *)a2;
  *(_BYTE *)(result + 4) = *(_BYTE *)(a2 + 4);
  return result;
}

