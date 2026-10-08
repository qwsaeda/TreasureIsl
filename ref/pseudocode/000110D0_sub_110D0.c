/* ea:   0x000110D0 */
/* name: sub_110D0 */
/* size: 64 */
int __fastcall sub_110D0(_DWORD *a1, int a2, int a3, int a4)
{
  int result; // r0

  a1[1] = a2;
  if ( a2 == 0 )
    __debugbreak();
  a1[2] = a3 - *(_DWORD *)(a2 + 4) + 64;
  result = *(_DWORD *)(a2 + 8);
  a1[3] = a4 - result + 32;
  return result;
}

