/* ea:   0x000450F8 */
/* name: sub_450F8 */
/* size: 76 */
int __fastcall sub_450F8(_DWORD *a1, unsigned int a2, int a3)
{
  unsigned int v4; // r2

  v4 = (a1[1] - *a1) >> 2;
  if ( a2 >= v4 )
    return sub_45234(a1, a1[1], a2 - v4, a3);
  else
    return sub_45144(a1, *a1 + 4 * a2, a1[1], a3);
}

