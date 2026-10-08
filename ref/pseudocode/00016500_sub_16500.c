/* ea:   0x00016500 */
/* name: sub_16500 */
/* size: 76 */
int __fastcall sub_16500(unsigned __int16 *a1, __int16 a2)
{
  unsigned __int16 v3; // r3
  unsigned __int16 v4; // r0

  v3 = a1[1];
  v4 = *a1 + a2;
  *a1 = v4;
  if ( v4 > (unsigned int)v3 )
    *a1 = v3;
  return *a1;
}

