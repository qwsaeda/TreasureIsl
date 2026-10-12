/* ea:   0x00066A04 */
/* name: sub_66A04 */
/* size: 72 */
int __fastcall sub_66A04(int a1)
{
  int v2; // r1
  int v3; // r4

  if ( *(_DWORD *)(a1 + 32) == *(_DWORD *)(a1 + 36) )
    return sub_65124();
  v2 = *(_DWORD *)(a1 + 36);
  v3 = *(_DWORD *)(v2 - 4);
  sub_66A4C(a1 + 32, (void *)(v2 - 4));
  return v3;
}

