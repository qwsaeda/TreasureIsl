/* ea:   0x00038934 */
/* name: sub_38934 */
/* size: 64 */
_DWORD *__fastcall sub_38934(_DWORD *a1, int a2, _DWORD *a3)
{
  int i; // r5

  for ( i = a2; i != 0; a1 += 3 )
  {
    if ( a1 != nullptr )
      sub_132FC(a1, a3);
    --i;
  }
  return a1;
}

