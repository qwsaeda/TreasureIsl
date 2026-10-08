/* ea:   0x00054638 */
/* name: sub_54638 */
/* size: 68 */
char *__fastcall sub_54638(int a1, int a2, int a3, int a4)
{
  char *v6; // r0
  char *v7; // r7
  char *v8; // r4

  v6 = (char *)sub_5445C(a1 + 8, a2);
  v7 = v6;
  if ( a3 != a4 )
  {
    v8 = &v6[-a3];
    do
    {
      if ( &v8[a3] != nullptr )
        sub_5454C((int)&v8[a3], a3);
      a3 += 36;
    }
    while ( a3 != a4 );
  }
  return v7;
}

