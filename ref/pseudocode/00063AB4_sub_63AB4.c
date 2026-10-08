/* ea:   0x00063AB4 */
/* name: sub_63AB4 */
/* size: 44 */
_DWORD *__fastcall sub_63AB4(_DWORD *result, char a2)
{
  _DWORD *v2; // r4

  v2 = result;
  *result = &off_7BCA8;
  if ( (a2 & 1) != 0 )
  {
    operator delete(result);
    return v2;
  }
  return result;
}

