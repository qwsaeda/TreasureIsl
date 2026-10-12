/* ea:   0x000615D4 */
/* name: sub_615D4 */
/* size: 172 */
_DWORD *__fastcall sub_615D4(_DWORD *result, int a2, int a3, unsigned __int16 *a4, unsigned __int16 *a5)
{
  unsigned __int16 *v7; // r3

  while ( 1 )
  {
    if ( a2 == a3 )
    {
      *result = a3;
      return result;
    }
    v7 = a4;
    if ( a4 != a5 )
      break;
LABEL_5:
    a2 -= 2;
  }
  while ( *(unsigned __int16 *)(a2 - 2) != *v7 )
  {
    if ( ++v7 == a5 )
      goto LABEL_5;
  }
  *result = a2;
  return result;
}

