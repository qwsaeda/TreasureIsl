/* ea:   0x000607B0 */
/* name: sub_607B0 */
/* size: 116 */
int __fastcall sub_607B0(_DWORD *a1, int a2, unsigned int a3)
{
  int v4; // r5
  int v5; // r3
  char v7; // [sp+0h] [bp-18h] BYREF

  v4 = a1[1];
  if ( a3 >= (v4 - *a1) >> 1 )
    return -1;
  v5 = sub_619FC(*a1 + 2 * a3, v4, a2, &v7);
  if ( v5 == a1[1] )
    return -1;
  else
    return (v5 - *a1) >> 1;
}

