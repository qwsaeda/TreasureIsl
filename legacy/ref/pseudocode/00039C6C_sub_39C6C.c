/* ea:   0x00039C6C */
/* name: sub_39C6C */
/* size: 28 */
int __fastcall sub_39C6C(int result, int a2)
{
  *(_DWORD *)result = *(_DWORD *)a2;
  *(_BYTE *)(result + 4) = *(_BYTE *)(a2 + 4);
  return result;
}

