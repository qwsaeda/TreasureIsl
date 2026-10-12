/* ea:   0x0004E0FC */
/* name: sub_4E0FC */
/* size: 60 */
_DWORD *__fastcall sub_4E0FC(int a1, int a2, _DWORD *a3)
{
  int v4; // r2
  int v5; // r1

  v4 = (a2 - a1) >> 2;
  if ( v4 > 0 )
  {
    v5 = a1 - (_DWORD)a3;
    do
    {
      --v4;
      *a3 = *(_DWORD *)((char *)a3 + v5);
      ++a3;
    }
    while ( v4 != 0 );
  }
  return a3;
}

