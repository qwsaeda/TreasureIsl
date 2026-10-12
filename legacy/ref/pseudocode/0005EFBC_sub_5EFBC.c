/* ea:   0x0005EFBC */
/* name: sub_5EFBC */
/* size: 180 */
int __fastcall sub_5EFBC(int result, int a2, _WORD *a3, int a4, int a5, int (__fastcall ***a6)(_DWORD))
{
  int v6; // r10
  int i; // r8
  int v10; // t1
  unsigned __int8 v11; // r0

  v6 = result;
  for ( i = a2; i != v6; ++a3 )
  {
    v10 = *(unsigned __int8 *)--i;
    result = v10 != *(unsigned __int8 *)(a4 + 1);
    if ( v10 != *(unsigned __int8 *)(a4 + 1) )
    {
      v11 = (**a6)(a6);
      result = (unsigned __int16)(8 * (*(_BYTE *)(3 * v11 + a5 + 1) & 0xFC | (32 * (*(_BYTE *)(3 * v11 + a5) & 0xF8))))
             | (*(unsigned __int8 *)(3 * v11 + a5 + 2) >> 3);
      *a3 = result;
    }
  }
  return result;
}

