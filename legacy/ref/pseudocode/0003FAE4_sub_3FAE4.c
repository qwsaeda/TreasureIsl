/* ea:   0x0003FAE4 */
/* name: sub_3FAE4 */
/* size: 56 */
_DWORD *__fastcall sub_3FAE4(_DWORD *a1, char a2)
{
  _DWORD *result; // r0

  *a1 = &off_6F47C;
  sub_43C20(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

