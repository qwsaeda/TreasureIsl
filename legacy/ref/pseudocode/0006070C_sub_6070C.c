/* ea:   0x0006070C */
/* name: sub_6070C */
/* size: 164 */
unsigned int __fastcall sub_6070C(int a1, int *a2)
{
  unsigned int result; // r0
  int v4; // r0
  unsigned int v5; // r0
  int v6; // r0

  result = *a2 == a2[1];
  if ( *a2 != a2[1] )
  {
    v4 = sub_12F04(asc_7FBA4);
    v5 = sub_609C0(a2, asc_7FBA4, -1, v4);
    if ( v5 != -1 )
      sub_13110(a2, v5, -1);
    v6 = sub_12F04(asc_7FBA0);
    result = sub_609C0(a2, asc_7FBA0, -1, v6);
    if ( result != -1 )
      return (unsigned int)sub_13110(a2, result, -1);
  }
  return result;
}

