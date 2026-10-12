/* ea:   0x0005360C */
/* name: sub_5360C */
/* size: 52 */
_BYTE *__fastcall sub_5360C(char *a1, int a2, _BYTE *a3)
{
  char *v3; // r3
  int v5; // r2
  char v6; // t1

  v3 = a1;
  v5 = a2 - (_DWORD)a1;
  if ( a2 - (int)a1 > 0 )
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

