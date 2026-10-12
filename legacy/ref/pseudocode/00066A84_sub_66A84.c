/* ea:   0x00066A84 */
/* name: sub_66A84 */
/* size: 72 */
_BYTE *__fastcall sub_66A84(int a1)
{
  int v2; // r1
  int v3; // r4

  if ( *(_DWORD *)(a1 + 20) == *(_DWORD *)(a1 + 24) )
    return sub_64EB8();
  v2 = *(_DWORD *)(a1 + 24);
  v3 = *(_DWORD *)(v2 - 4);
  sub_66ACC(a1 + 20, (void *)(v2 - 4));
  return (_BYTE *)v3;
}

