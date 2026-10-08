/* ea:   0x0005EC44 */
/* name: sub_5EC44 */
/* size: 136 */
int __fastcall sub_5EC44(int result, int a2, _WORD *a3, int a4, int (__fastcall ***a5)(_DWORD, int, int, _DWORD))
{
  int v5; // r9
  int i; // r8
  int v9; // t1
  unsigned __int8 v10; // r0

  v5 = result;
  for ( i = a2; i != v5; ++a3 )
  {
    v9 = *(unsigned __int8 *)--i;
    v10 = (**a5)(a5, v9, a4, (unsigned __int16)*a3);
    result = (unsigned __int16)(8 * (*(_BYTE *)(3 * v10 + a4 + 1) & 0xFC | (32 * (*(_BYTE *)(3 * v10 + a4) & 0xF8))))
           | (*(unsigned __int8 *)(3 * v10 + a4 + 2) >> 3);
    *a3 = result;
  }
  return result;
}

