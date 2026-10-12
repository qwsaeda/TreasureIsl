/* ea:   0x00019AE0 */
/* name: sub_19AE0 */
/* size: 44 */
_DWORD *__fastcall sub_19AE0(_DWORD *result, char a2)
{
  _DWORD *v2; // r4

  v2 = result;
  *result = &off_6C910;
  if ( (a2 & 1) != 0 )
  {
    operator delete(result);
    return v2;
  }
  return result;
}

