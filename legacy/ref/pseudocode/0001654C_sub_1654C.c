/* ea:   0x0001654C */
/* name: sub_1654C */
/* size: 36 */
unsigned int __fastcall sub_1654C(unsigned int *a1, int a2)
{
  unsigned int v2; // r2
  unsigned int v3; // r1

  v2 = a1[1];
  v3 = *a1 + a2;
  *a1 = v3;
  if ( v3 > v2 )
    *a1 = v2;
  return *a1;
}

