/* ea:   0x00032410 */
/* name: sub_32410 */
/* size: 104 */
unsigned int __fastcall sub_32410(unsigned __int16 *a1, __int16 a2)
{
  unsigned __int16 v2; // r3
  unsigned __int16 *v3; // r5
  unsigned __int16 v4; // r0
  unsigned __int16 v5; // r3
  int v6; // r0
  unsigned int result; // r0

  v2 = *a1;
  v3 = a1 + 2;
  v4 = a1[2];
  v3[1] = v2;
  if ( v4 > (unsigned int)v2 )
    *v3 = v2;
  v5 = v3[1];
  v6 = (unsigned __int16)(*v3 + a2);
  *v3 = v6;
  result = v6 << 16;
  if ( HIWORD(result) > v5 )
    *v3 = v5;
  return result;
}

