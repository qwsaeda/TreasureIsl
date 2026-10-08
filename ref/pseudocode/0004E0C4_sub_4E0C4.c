/* ea:   0x0004E0C4 */
/* name: sub_4E0C4 */
/* size: 56 */
_DWORD *__fastcall sub_4E0C4(int *a1, int a2, _DWORD *a3)
{
  int *v3; // r3
  int v5; // r2
  int v6; // t1

  v3 = a1;
  v5 = (a2 - (int)a1) >> 2;
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

