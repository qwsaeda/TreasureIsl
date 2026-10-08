/* ea:   0x00020060 */
/* name: sub_20060 */
/* size: 40 */
_DWORD *__fastcall sub_20060(_DWORD *a1, char a2)
{
  _DWORD *result; // r0

  sub_1B2AC(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

