/* ea:   0x000186B8 */
/* name: sub_186B8 */
/* size: 56 */
_WORD *__fastcall sub_186B8(__int16 *a1, int a2, _WORD *a3)
{
  __int16 *v3; // r3
  int v5; // r2
  __int16 v6; // t1

  v3 = a1;
  v5 = (a2 - (int)a1) >> 1;
  if ( v5 > 0 )
  {
    do
    {
      v6 = *v3++;
      --v5;
      *a3++ = v6;
    }
    while ( v5 != 0 );
  }
  return a3;
}

