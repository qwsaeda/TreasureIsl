/* ea:   0x0006261C */
/* name: sub_6261C */
/* size: 40 */
_DWORD *__fastcall sub_6261C(_DWORD *a1, char a2)
{
  _DWORD *result; // r0

  sub_625E0(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

