/* ea:   0x000653B0 */
/* name: sub_653B0 */
/* size: 84 */
_DWORD *__fastcall sub_653B0(_DWORD *a1, char a2)
{
  _DWORD *v4; // r4
  _DWORD *result; // r0

  a1[1] = off_6D220;
  v4 = a1 + 1;
  sub_620B0((int)(a1 + 1));
  *v4 = &off_6D248;
  *a1 = off_7BD34;
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

