/* ea:   0x0001E9EC */
/* name: sub_1E9EC */
/* size: 40 */
_DWORD *__fastcall sub_1E9EC(_DWORD *a1, char a2)
{
  _DWORD *result; // r0

  sub_1E798(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

