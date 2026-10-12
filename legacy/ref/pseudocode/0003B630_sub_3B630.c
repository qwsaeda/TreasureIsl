/* ea:   0x0003B630 */
/* name: sub_3B630 */
/* size: 60 */
_WORD *__fastcall sub_3B630(int a1, int a2, _WORD *a3)
{
  int v4; // r2
  int v5; // r1

  v4 = (a2 - a1) >> 1;
  if ( v4 > 0 )
  {
    v5 = a1 - (_DWORD)a3;
    do
    {
      --v4;
      *a3 = *(_WORD *)((char *)a3 + v5);
      ++a3;
    }
    while ( v4 != 0 );
  }
  return a3;
}

