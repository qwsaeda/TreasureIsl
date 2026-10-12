/* ea:   0x0005EB58 */
/* name: sub_5EB58 */
/* size: 136 */
int __fastcall sub_5EB58(
        int result,
        unsigned __int8 *a2,
        _WORD *a3,
        int a4,
        int (__fastcall ***a5)(_DWORD, int, int, _DWORD))
{
  unsigned __int8 *i; // r8
  int v9; // t1
  unsigned __int8 v10; // r0

  for ( i = (unsigned __int8 *)result; i != a2; ++a3 )
  {
    v9 = *i++;
    v10 = (**a5)(a5, v9, a4, (unsigned __int16)*a3);
    result = (unsigned __int16)(8 * (*(_BYTE *)(3 * v10 + a4 + 1) & 0xFC | (32 * (*(_BYTE *)(3 * v10 + a4) & 0xF8))))
           | (*(unsigned __int8 *)(3 * v10 + a4 + 2) >> 3);
    *a3 = result;
  }
  return result;
}

