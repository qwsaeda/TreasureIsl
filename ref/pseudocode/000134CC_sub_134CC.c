/* ea:   0x000134CC */
/* name: sub_134CC */
/* size: 88 */
_WORD *__fastcall sub_134CC(_DWORD *a1, int a2, int a3)
{
  _WORD *result; // r0
  char v7; // [sp+0h] [bp-14h] BYREF

  sub_13850(a1, ((a3 - a2) >> 1) + 1);
  result = (_WORD *)sub_13524(a2, a3, *a1, &v7);
  a1[1] = result;
  *result = 0;
  return result;
}

