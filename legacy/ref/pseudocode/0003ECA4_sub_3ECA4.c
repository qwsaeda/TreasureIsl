/* ea:   0x0003ECA4 */
/* name: sub_3ECA4 */
/* size: 56 */
_DWORD *__fastcall sub_3ECA4(_DWORD *a1, char a2)
{
  _DWORD *result; // r0

  *a1 = off_6F458;
  sub_4ECFC(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

