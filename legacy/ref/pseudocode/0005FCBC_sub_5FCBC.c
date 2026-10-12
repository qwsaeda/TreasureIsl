/* ea:   0x0005FCBC */
/* name: sub_5FCBC */
/* size: 108 */
char *__fastcall sub_5FCBC(int a1, char *a2, int a3, _WORD *a4, int *a5)
{
  char *v7; // r5
  int v8; // r3
  int v9; // r2
  __int16 v10; // t1

  v7 = a2;
  v8 = *a5 - 1;
  if ( *a5 != 1 )
  {
    v9 = 0;
    do
    {
      if ( a3 <= v9 >> 1 )
        break;
      v10 = *(_WORD *)v7;
      v7 += 2;
      v9 += 2;
      *a4++ = v10;
      if ( v10 == 10 )
        break;
      --v8;
    }
    while ( v8 != 0 );
  }
  *a4 = 0;
  *a5 = (v7 - a2) >> 1;
  return (char *)(v7 - a2);
}

