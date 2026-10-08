/* ea:   0x0001E728 */
/* name: sub_1E728 */
/* size: 68 */
_DWORD *__fastcall sub_1E728(_DWORD *a1, char a2)
{
  _DWORD *result; // r0

  *a1 = off_6D220;
  sub_620B0(a1);
  *a1 = &off_6D248;
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

