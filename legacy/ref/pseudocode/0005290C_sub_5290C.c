/* ea:   0x0005290C */
/* name: sub_5290C */
/* size: 76 */
int __fastcall sub_5290C(int *a1, unsigned int a2)
{
  int v3; // r0

  v3 = *a1;
  if ( a2 >= a1[1] - v3 )
    return sub_52B98((int)a1, (void *)a1[1]);
  else
    return sub_529D4((int)a1, (void *)(v3 + a2), (void *)a1[1]);
}

