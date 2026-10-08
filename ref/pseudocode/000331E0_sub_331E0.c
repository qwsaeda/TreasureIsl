/* ea:   0x000331E0 */
/* name: sub_331E0 */
/* size: 76 */
_DWORD *__fastcall sub_331E0(_DWORD *a1, unsigned int a2, int a3)
{
  unsigned int v4; // r2

  v4 = (a1[1] - *a1) >> 2;
  if ( a2 >= v4 )
    return (_DWORD *)sub_364E0(a1, a1[1], a2 - v4, a3);
  else
    return sub_305A8((int)a1, (_DWORD *)(*a1 + 4 * a2), a1[1]);
}

