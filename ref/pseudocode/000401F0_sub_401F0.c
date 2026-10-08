/* ea:   0x000401F0 */
/* name: sub_401F0 */
/* size: 132 */
_DWORD *__fastcall sub_401F0(_DWORD *a1, int a2, int a3)
{
  char v7; // [sp+0h] [bp-18h] BYREF

  sub_4EC88(a1);
  a1[12] = 0;
  a1[13] = 0;
  a1[14] = 0;
  sub_3326C(a1 + 15, (int)&v7, 0);
  a1[16] = 0;
  a1[17] = 0;
  a1[18] = 0;
  a1[19] = 0;
  a1[20] = 0;
  a1[21] = 0;
  a1[22] = 0;
  *a1 = off_6F498;
  a1[23] = 0;
  a1[24] = a2;
  a1[25] = a3;
  return a1;
}

