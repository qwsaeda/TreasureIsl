/* ea:   0x000199F0 */
/* name: sub_199F0 */
/* size: 60 */
_DWORD *__fastcall sub_199F0(_DWORD *a1, char a2)
{
  _DWORD *result; // r0

  *a1 = &off_6C90C;
  sub_19A2C(a1 + 1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

