/* ea:   0x0004253C */
/* name: sub_4253C */
/* size: 56 */
_DWORD *__fastcall sub_4253C(_DWORD *a1, char a2)
{
  _DWORD *result; // r0

  *a1 = off_6F410;
  sub_4ECFC(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

