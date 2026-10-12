/* ea:   0x0004AC18 */
/* name: sub_4AC18 */
/* size: 72 */
int __fastcall sub_4AC18(int a1)
{
  unsigned int v2; // r2
  int result; // r0

  v2 = *(_DWORD *)(a1 + 8) - 1;
  *(_DWORD *)(a1 + 8) = v2;
  result = v2 - ((613566757 * (unsigned __int64)v2) >> 32);
  if ( v2 / 7 < v2 / 0xA )
  {
    *(_DWORD *)(a1 + 28) = 0;
    return 0;
  }
  return result;
}

