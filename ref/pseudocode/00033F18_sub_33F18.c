/* ea:   0x00033F18 */
/* name: sub_33F18 */
/* size: 132 */
unsigned int __fastcall sub_33F18(_DWORD *a1)
{
  unsigned int v2; // r4
  unsigned int v3; // r3
  unsigned int result; // r0
  unsigned int v5; // r3

  v2 = 0;
  if ( a1[7] == a1[8] )
  {
    v5 = a1[12];
    result = ((a1[5] - a1[4]) >> 2) - 1;
    if ( v5 < result )
      v2 = v5 + 1;
    a1[12] = v2;
  }
  else
  {
    v3 = a1[13];
    result = ((a1[8] - a1[7]) >> 2) - 1;
    if ( v3 < result )
      v2 = v3 + 1;
    a1[13] = v2;
  }
  return result;
}

