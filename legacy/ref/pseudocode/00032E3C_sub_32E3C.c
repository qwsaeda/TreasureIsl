/* ea:   0x00032E3C */
/* name: sub_32E3C */
/* size: 108 */
int __fastcall sub_32E3C(int *a1, int a2)
{
  int v4; // r0

  if ( a2 < 1 || a2 > 6 )
  {
    v4 = rand();
    a1[2] = 24;
    *a1 = v4 % 6;
    return v4 % 6 + 1;
  }
  else
  {
    *a1 = a2 - 1;
    a1[2] = 24;
    return *a1 + 1;
  }
}

