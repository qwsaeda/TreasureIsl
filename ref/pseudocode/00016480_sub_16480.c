/* ea:   0x00016480 */
/* name: sub_16480 */
/* size: 84 */
_DWORD *__fastcall sub_16480(_DWORD *a1, _WORD *a2)
{
  int v4; // r0
  char v6; // [sp+0h] [bp-10h] BYREF

  *a1 = 0;
  a1[1] = 0;
  a1[2] = 0;
  v4 = sub_12F04(a2);
  sub_183A4(a1, a2, &a2[v4], &v6);
  return a1;
}

