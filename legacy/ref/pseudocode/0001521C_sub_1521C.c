/* ea:   0x0001521C */
/* name: sub_1521C */
/* size: 56 */
_DWORD *__fastcall sub_1521C(_DWORD *a1, char a2)
{
  _DWORD *result; // r0

  *a1 = &off_6C834;
  sub_16B24(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

