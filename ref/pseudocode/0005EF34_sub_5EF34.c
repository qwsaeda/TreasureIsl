/* ea:   0x0005EF34 */
/* name: sub_5EF34 */
/* size: 136 */
int __fastcall sub_5EF34(int result, int a2, _WORD *a3, int a4, int a5, int a6)
{
  int v6; // r10
  int i; // r8
  int v10; // t1

  v6 = result;
  for ( i = a2; i != v6; ++a3 )
  {
    v10 = *(unsigned __int8 *)--i;
    result = v10 != *(unsigned __int8 *)(a4 + 1);
    if ( v10 != *(unsigned __int8 *)(a4 + 1) )
    {
      result = *(unsigned __int16 *)(a5 + 2 * (*(unsigned __int8 (__fastcall **)(int))(*(_DWORD *)a6 + 4))(a6));
      *a3 = result;
    }
  }
  return result;
}

