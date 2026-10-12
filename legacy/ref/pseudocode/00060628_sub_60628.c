/* ea:   0x00060628 */
/* name: sub_60628 */
/* size: 104 */
unsigned int __fastcall sub_60628(int a1, int *a2)
{
  unsigned int result; // r0
  int v4; // r0

  result = *a2 == a2[1];
  if ( *a2 != a2[1] )
  {
    v4 = sub_12F04(asc_7FB98);
    result = sub_60824(a2, asc_7FB98, 0, v4);
    if ( result != -1 )
      return (unsigned int)sub_13110(a2, result, -1);
  }
  return result;
}

