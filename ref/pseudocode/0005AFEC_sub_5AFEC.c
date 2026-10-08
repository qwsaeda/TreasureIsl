/* ea:   0x0005AFEC */
/* name: sub_5AFEC */
/* size: 96 */
unsigned int __fastcall sub_5AFEC(_DWORD *a1, unsigned int a2)
{
  int v3; // r2
  int v4; // r4
  int v5; // r5
  unsigned int result; // r0

  sub_5AEAC(a1, a2);
  v3 = a1[37];
  v4 = a1[38];
  v5 = a1[39];
  a1[40] = (unsigned int)(a1[36] * a1[3]) >> 6;
  a1[41] = (unsigned int)(v3 * a1[3]) >> 6;
  a1[42] = (unsigned int)(v4 * a1[3]) >> 6;
  result = (unsigned int)(v5 * a1[3]) >> 6;
  a1[43] = result;
  return result;
}

