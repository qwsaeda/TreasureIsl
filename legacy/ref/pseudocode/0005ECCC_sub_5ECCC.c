/* ea:   0x0005ECCC */
/* name: sub_5ECCC */
/* size: 148 */
int __fastcall sub_5ECCC(int result, unsigned __int8 *a2, _WORD *a3, int a4, int a5)
{
  unsigned __int8 *i; // r6
  int v8; // r1

  for ( i = (unsigned __int8 *)result; i != a2; ++a3 )
  {
    v8 = *i;
    result = v8 != *(unsigned __int8 *)(a4 + 1);
    if ( v8 != *(unsigned __int8 *)(a4 + 1) )
    {
      result = (unsigned __int16)(8 * (*(_BYTE *)(3 * v8 + a5 + 1) & 0xFC | (32 * (*(_BYTE *)(3 * v8 + a5) & 0xF8))))
             | (*(unsigned __int8 *)(3 * v8 + a5 + 2) >> 3);
      *a3 = result;
    }
    ++i;
  }
  return result;
}

