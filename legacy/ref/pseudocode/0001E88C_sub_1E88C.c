/* ea:   0x0001E88C */
/* name: sub_1E88C */
/* size: 88 */
char *__fastcall sub_1E88C(char *a1, int a2, unsigned int a3)
{
  char *v3; // r5
  char *v5; // r3
  int v6; // r1
  char *v7; // r4

  v3 = a1;
  v5 = (char *)(a2 - 16);
  if ( (unsigned int)a1 <= a2 - 16 )
  {
    do
    {
      v6 = (v5 - v3) >> 5;
      v7 = &v3[16 * v6];
      if ( *(_DWORD *)v7 < a3 )
      {
        v3 = v7 + 16;
      }
      else
      {
        if ( *(_DWORD *)v7 <= a3 )
          return &v3[16 * v6];
        v5 = v7 - 16;
      }
    }
    while ( v3 <= v5 );
  }
  return (char *)a2;
}

