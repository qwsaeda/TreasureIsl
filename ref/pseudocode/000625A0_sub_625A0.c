/* ea:   0x000625A0 */
/* name: sub_625A0 */
/* size: 64 */
_DWORD *__fastcall sub_625A0(_DWORD *a1, char a2)
{
  _DWORD *result; // r0

  *a1 = off_7BC2C;
  sub_623D8((int)a1);
  sub_625E0(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

