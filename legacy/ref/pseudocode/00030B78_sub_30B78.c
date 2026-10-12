/* ea:   0x00030B78 */
/* name: sub_30B78 */
/* size: 76 */
int __fastcall sub_30B78(int a1, unsigned int a2, int a3)
{
  unsigned int v4; // r2

  v4 = (*(_DWORD *)(a1 + 4) - *(_DWORD *)a1) >> 2;
  if ( a2 >= v4 )
    return sub_36EA4(a1, *(_DWORD *)(a1 + 4), a2 - v4, a3);
  else
    return sub_3322C(a1, (void *)(*(_DWORD *)a1 + 4 * a2), *(void **)(a1 + 4));
}

