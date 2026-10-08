/* ea:   0x000111B4 */
/* name: sub_111B4 */
/* size: 44 */
_DWORD *__fastcall sub_111B4(_DWORD *result, char a2)
{
  _DWORD *v2; // r4

  v2 = result;
  *result = &off_6B030;
  if ( (a2 & 1) != 0 )
  {
    operator delete(result);
    return v2;
  }
  return result;
}

