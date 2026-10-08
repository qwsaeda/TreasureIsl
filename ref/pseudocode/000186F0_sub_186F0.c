/* ea:   0x000186F0 */
/* name: sub_186F0 */
/* size: 20 */
int __fastcall sub_186F0(int a1, int a2)
{
  int result; // r0

  result = *(_DWORD *)(a1 + 276);
  if ( a2 <= 4 )
    result += 12 * a2;
  return result;
}

