/* ea:   0x0005B04C */
/* name: sub_5B04C */
/* size: 56 */
int __fastcall sub_5B04C(_DWORD *a1, unsigned int a2)
{
  unsigned int v3; // r1
  int v4; // r3
  int result; // r0

  sub_5AF6C((int)a1, a2);
  v3 = (255 - a1[4]) << 6;
  v4 = (v3 >> 8) + 64;
  result = 192 - (v3 >> 8);
  a1[64] = v4;
  a1[65] = result;
  a1[66] = result;
  a1[67] = v4;
  return result;
}

