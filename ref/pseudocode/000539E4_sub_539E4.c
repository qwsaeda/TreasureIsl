/* ea:   0x000539E4 */
/* name: sub_539E4 */
/* size: 104 */
int __fastcall sub_539E4(int a1, unsigned __int8 a2, unsigned __int8 a3, int a4)
{
  int i; // r6

  for ( i = *(_DWORD *)(a1 + 16); i != *(_DWORD *)(a1 + 20); i += 36 )
  {
    if ( a2 != 0
      && *(_DWORD *)(*(_DWORD *)(i + 24) + 4 * a2 - 4) == 0x10000
      && *(unsigned __int8 *)(i + 16) <= (unsigned int)a3 )
    {
      sub_540B4(a4, i);
    }
  }
  return 1;
}

