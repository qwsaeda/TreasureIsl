/* ea:   0x0001333C */
/* name: sub_1333C */
/* size: 64 */
_DWORD *__fastcall sub_1333C(_DWORD *a1)
{
  _WORD *v2; // r3

  *a1 = 0;
  a1[1] = 0;
  a1[2] = 0;
  v2 = (_WORD *)sub_1343C(a1 + 2, 8);
  *a1 = v2;
  a1[2] = v2 + 8;
  a1[1] = v2;
  *v2 = 0;
  return a1;
}

