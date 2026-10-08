/* ea:   0x0003053C */
/* name: sub_3053C */
/* size: 108 */
int __fastcall sub_3053C(unsigned __int16 *a1, __int16 a2)
{
  unsigned __int16 *v3; // r4
  unsigned __int16 v4; // r3
  unsigned __int16 v5; // r0
  int v6; // r3
  unsigned int v7; // r2
  int result; // r0

  v3 = a1 + 2;
  v4 = a1[1];
  v5 = *a1 + a2;
  *a1 = v5;
  if ( v5 > (unsigned int)v4 )
    *a1 = v4;
  v6 = *a1;
  v7 = *v3;
  v3[1] = v6;
  result = v6 << 16;
  if ( v7 > (unsigned __int16)v6 )
    *v3 = v6;
  return result;
}

