/* ea:   0x00060690 */
/* name: sub_60690 */
/* size: 124 */
int *__fastcall sub_60690(int a1, int *a2)
{
  int *result; // r0
  int v4; // r0

  result = (int *)(*a2 == a2[1]);
  if ( *a2 != a2[1] )
  {
    v4 = sub_60A84(a2, 32, 0);
    if ( v4 != -1 )
      sub_13110(a2, 0, v4);
    result = (int *)sub_60AF8(a2, 32, -1);
    if ( result != (int *)-1 )
      return sub_13110(a2, (unsigned int)result + 1, -1);
  }
  return result;
}

