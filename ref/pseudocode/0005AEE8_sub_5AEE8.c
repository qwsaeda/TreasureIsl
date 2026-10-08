/* ea:   0x0005AEE8 */
/* name: sub_5AEE8 */
/* size: 132 */
int __fastcall sub_5AEE8(int a1, unsigned int a2)
{
  int v2; // r1

  *(_DWORD *)(a1 + 36) = a2;
  if ( a2 <= 0x80 )
  {
    if ( a2 < 0x80 )
      return sub_5AED8(a1, (*(_DWORD *)(a1 + 24) << 16) / ((256 - a2) << 9));
    v2 = *(_DWORD *)(a1 + 24);
  }
  else
  {
    v2 = (__int64)(*(unsigned int *)(a1 + 24) * (unsigned __int64)(a2 << 9)) >> 16;
  }
  return sub_5AED8(a1, v2);
}

