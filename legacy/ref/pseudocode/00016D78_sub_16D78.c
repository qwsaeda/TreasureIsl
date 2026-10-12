/* ea:   0x00016D78 */
/* name: sub_16D78 */
/* size: 148 */
int __fastcall sub_16D78(unsigned __int16 *a1, unsigned __int16 a2)
{
  unsigned int v3; // r3
  unsigned int v4; // r3
  unsigned int v5; // r2
  int v6; // r3
  unsigned __int16 v7; // r0
  unsigned int v8; // r2
  int result; // r0

  v3 = *a1;
  if ( a2 < v3 )
    *a1 = v3 - a2;
  else
    *a1 = 0;
  v4 = sub_16D1C((int)(a1 + 8));
  v5 = *a1;
  if ( v4 > v5 )
    sub_16E0C(a1 + 8, v4 - v5);
  v6 = *a1;
  v7 = a1[2];
  a1[3] = v6;
  v8 = v7;
  result = v6 << 16;
  if ( v8 > (unsigned __int16)v6 )
    a1[2] = v6;
  return result;
}

