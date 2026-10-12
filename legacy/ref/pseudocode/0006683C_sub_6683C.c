/* ea:   0x0006683C */
/* name: sub_6683C */
/* size: 132 */
_DWORD *__fastcall sub_6683C(_DWORD *a1, char *a2, int a3)
{
  char v7; // [sp+0h] [bp-14h] BYREF
  char v8; // [sp+1h] [bp-13h] BYREF

  sub_1333C(a1 + 1);
  a1[4] = a3;
  *a1 = off_7BD0C;
  sub_65FE4((int)a1, a2);
  a1[5] = 0;
  a1[6] = 0;
  a1[7] = 0;
  a1[8] = 0;
  a1[9] = 0;
  a1[10] = 0;
  sub_668C0(a1 + 11, &v8, &v7);
  *a1 = off_7BD20;
  return a1;
}

