/* ea:   0x00017B98 */
/* name: sub_17B98 */
/* size: 48 */
int __fastcall sub_17B98(int a1)
{
  int *v1; // r2
  int *v2; // r1
  int v3; // r3
  int v4; // r0

  v1 = *(int **)(a1 + 28);
  v2 = *(int **)(a1 + 32);
  v3 = 0;
  while ( v1 != v2 )
  {
    v4 = *v1++;
    if ( v4 != 0 )
      ++v3;
  }
  return v3;
}

