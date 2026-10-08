/* ea:   0x0005350C */
/* name: sub_5350C */
/* size: 100 */
int __fastcall sub_5350C(int a1)
{
  unsigned int v2; // r0
  unsigned __int8 v3; // r1
  int v4; // r0

  v2 = *(unsigned __int8 *)(a1 + 49);
  if ( v2 >= 4 )
    return 0;
  v3 = v2 + 1;
  v4 = *(unsigned __int8 *)(a1 + 50);
  *(_BYTE *)(a1 + 49) = v3;
  if ( v4 == 1 && v3 > 1u )
    sub_17C7C((int)off_811A0, 3);
  return 1;
}

