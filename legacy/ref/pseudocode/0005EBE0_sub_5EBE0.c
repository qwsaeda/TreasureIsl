/* ea:   0x0005EBE0 */
/* name: sub_5EBE0 */
/* size: 100 */
unsigned int __fastcall sub_5EBE0(unsigned int result, int a2, _WORD *a3, int a4)
{
  unsigned int v4; // r7
  int i; // r5
  int v8; // t1
  _BYTE *v9; // r3

  v4 = result;
  for ( i = a2; i != v4; ++a3 )
  {
    v8 = *(unsigned __int8 *)--i;
    v9 = (_BYTE *)(3 * v8 + a4);
    result = (unsigned __int8)v9[2];
    *a3 = (8 * (v9[1] & 0xFC | (32 * (*v9 & 0xF8)))) | (result >> 3);
  }
  return result;
}

