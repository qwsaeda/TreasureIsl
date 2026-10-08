/* ea:   0x00038080 */
/* name: sub_38080 */
/* size: 104 */
__int16 **__fastcall sub_38080(__int16 **a1, int a2, __int16 **a3)
{
  __int16 **v3; // r5
  int v5; // r4

  v3 = a1;
  v5 = (a2 - (int)a1) / 12;
  if ( v5 > 0 )
  {
    do
    {
      if ( v3 != a3 )
        sub_17890((int)a3, *v3, v3[1]);
      --v5;
      v3 += 3;
      a3 += 3;
    }
    while ( v5 != 0 );
  }
  return a3;
}

