/* ea:   0x0005ED60 */
/* name: sub_5ED60 */
/* size: 140 */
unsigned __int8 *__fastcall sub_5ED60(unsigned __int8 *result, unsigned __int8 *a2, _WORD *a3, int a4, int a5, int a6)
{
  unsigned __int8 *i; // r8

  for ( i = result; i != a2; ++a3 )
  {
    result = (unsigned __int8 *)(*i != *(unsigned __int8 *)(a4 + 1));
    if ( *i != *(unsigned __int8 *)(a4 + 1) )
    {
      result = (unsigned __int8 *)*(unsigned __int16 *)(a5
                                                      + 2
                                                      * (*(unsigned __int8 (__fastcall **)(int))(*(_DWORD *)a6 + 4))(a6));
      *a3 = (_WORD)result;
    }
    ++i;
  }
  return result;
}

