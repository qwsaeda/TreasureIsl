/* ea:   0x00060A84 */
/* name: sub_60A84 */
/* size: 116 */
int __fastcall sub_60A84(_DWORD *a1, int a2, unsigned int a3)
{
  int v4; // r5
  int v5; // r3
  char v7; // [sp+0h] [bp-18h] BYREF

  v4 = a1[1];
  if ( a3 > (v4 - *a1) >> 1 )
    return -1;
  v5 = sub_61B74(*a1 + 2 * a3, v4, a2, &v7);
  if ( v5 == a1[1] )
    return -1;
  else
    return (v5 - *a1) >> 1;
}

