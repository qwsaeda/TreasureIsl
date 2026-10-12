/* ea:   0x0001E7F0 */
/* name: sub_1E7F0 */
/* size: 44 */
_DWORD *__fastcall sub_1E7F0(_DWORD *result, char a2)
{
  _DWORD *v2; // r4

  v2 = result;
  *result = &off_6D270;
  if ( (a2 & 1) != 0 )
  {
    operator delete(result);
    return v2;
  }
  return result;
}

