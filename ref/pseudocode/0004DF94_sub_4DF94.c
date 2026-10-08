/* ea:   0x0004DF94 */
/* name: sub_4DF94 */
/* size: 32 */
unsigned int __fastcall sub_4DF94(int a1, unsigned int a2)
{
  unsigned int result; // r0

  result = *(_DWORD *)(a1 + 8);
  if ( a2 <= result )
  {
    result -= a2;
    *(_DWORD *)(a1 + 8) = result;
  }
  else
  {
    *(_DWORD *)(a1 + 8) = 0;
  }
  return result;
}

