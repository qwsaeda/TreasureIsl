/* ea:   0x0001E920 */
/* name: sub_1E920 */
/* size: 44 */
BOOL __fastcall sub_1E920(int a1, int a2)
{
  if ( a2 == 0 )
    __debugbreak();
  return *(_DWORD *)(a2 + 4) != *(_DWORD *)(a2 + 8);
}

