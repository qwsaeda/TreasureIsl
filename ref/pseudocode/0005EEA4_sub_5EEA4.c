/* ea:   0x0005EEA4 */
/* name: sub_5EEA4 */
/* size: 144 */
int __fastcall sub_5EEA4(int result, int a2, _WORD *a3, int a4, int a5)
{
  int v5; // r8
  int i; // r6
  int v8; // r1
  int v9; // t1

  v5 = result;
  for ( i = a2; i != v5; ++a3 )
  {
    v9 = *(unsigned __int8 *)--i;
    v8 = v9;
    result = v9 != *(unsigned __int8 *)(a4 + 1);
    if ( v9 != *(unsigned __int8 *)(a4 + 1) )
    {
      result = (unsigned __int16)(8 * (*(_BYTE *)(3 * v8 + a5 + 1) & 0xFC | (32 * (*(_BYTE *)(3 * v8 + a5) & 0xF8))))
             | (*(unsigned __int8 *)(3 * v8 + a5 + 2) >> 3);
      *a3 = result;
    }
  }
  return result;
}

