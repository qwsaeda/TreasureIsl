/* ea:   0x0005EAF4 */
/* name: sub_5EAF4 */
/* size: 100 */
unsigned __int8 *__fastcall sub_5EAF4(unsigned __int8 *result, unsigned __int8 *a2, _WORD *a3, int a4)
{
  unsigned __int8 *i; // r5
  int v7; // t1
  _BYTE *v8; // r3

  for ( i = result; i != a2; ++a3 )
  {
    v7 = *i++;
    v8 = (_BYTE *)(3 * v7 + a4);
    result = (unsigned __int8 *)(unsigned __int8)v8[2];
    *a3 = (8 * (v8[1] & 0xFC | (32 * (*v8 & 0xF8)))) | ((unsigned int)result >> 3);
  }
  return result;
}

