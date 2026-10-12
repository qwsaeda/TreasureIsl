/* ea:   0x00034844 */
/* name: sub_34844 */
/* size: 76 */
int __fastcall sub_34844(_DWORD *a1, unsigned int a2, int a3)
{
  unsigned int v4; // r2

  v4 = (a1[1] - *a1) >> 3;
  if ( a2 >= v4 )
    return sub_36A9C(a1, a1[1], a2 - v4, a3);
  else
    return sub_348F8(a1, *a1 + 8 * a2, a1[1], a3);
}

