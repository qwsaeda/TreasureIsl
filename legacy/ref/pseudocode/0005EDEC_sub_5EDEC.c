/* ea:   0x0005EDEC */
/* name: sub_5EDEC */
/* size: 184 */
int __fastcall sub_5EDEC(int result, unsigned __int8 *a2, _WORD *a3, int a4, int a5, int (__fastcall ***a6)(_DWORD))
{
  unsigned __int8 *i; // r8
  unsigned __int8 v10; // r0

  for ( i = (unsigned __int8 *)result; i != a2; ++a3 )
  {
    result = *i != *(unsigned __int8 *)(a4 + 1);
    if ( *i != *(unsigned __int8 *)(a4 + 1) )
    {
      v10 = (**a6)(a6);
      result = (unsigned __int16)(8 * (*(_BYTE *)(3 * v10 + a5 + 1) & 0xFC | (32 * (*(_BYTE *)(3 * v10 + a5) & 0xF8))))
             | (*(unsigned __int8 *)(3 * v10 + a5 + 2) >> 3);
      *a3 = result;
    }
    ++i;
  }
  return result;
}

