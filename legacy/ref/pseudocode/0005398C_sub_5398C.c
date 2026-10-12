/* ea:   0x0005398C */
/* name: sub_5398C */
/* size: 88 */
_DWORD *__fastcall sub_5398C(_DWORD *a1, int a2)
{
  _DWORD *v3; // r0
  char v5; // [sp+0h] [bp-10h] BYREF

  *a1 = a2;
  v3 = a1 + 1;
  *v3 = 0;
  v3[1] = 0;
  sub_54418(v3 + 2, &v5, 0);
  a1[4] = 0;
  a1[5] = 0;
  a1[6] = 0;
  a1[8] = 0;
  a1[9] = 0;
  a1[10] = 0;
  return a1;
}

