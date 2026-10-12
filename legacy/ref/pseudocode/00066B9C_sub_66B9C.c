/* ea:   0x00066B9C */
/* name: sub_66B9C */
/* size: 100 */
int __fastcall sub_66B9C(_DWORD *a1)
{
  int *v2; // r1
  int v3; // r0

  if ( a1 == nullptr )
    return -2;
  v2 = (int *)a1[7];
  if ( v2 == nullptr )
    return -2;
  a1[2] = 0;
  a1[5] = 0;
  a1[6] = 0;
  v3 = 7;
  if ( v2[3] == 0 )
    v3 = 0;
  *v2 = v3;
  sub_67370(*(_DWORD *)(a1[7] + 20), a1, 0);
  return 0;
}

