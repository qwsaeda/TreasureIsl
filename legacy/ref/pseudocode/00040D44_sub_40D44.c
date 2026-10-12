/* ea:   0x00040D44 */
/* name: sub_40D44 */
/* size: 56 */
_DWORD *__fastcall sub_40D44(_DWORD *a1, char a2)
{
  _DWORD *result; // r0

  *a1 = off_6F3EC;
  sub_4ECFC(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

