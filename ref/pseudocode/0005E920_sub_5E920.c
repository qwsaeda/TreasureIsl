/* ea:   0x0005E920 */
/* name: sub_5E920 */
/* size: 80 */
BOOL __fastcall sub_5E920(int a1)
{
  if ( a1 == 0 )
  {
    __debugbreak();
    __debugbreak();
  }
  if ( *(_DWORD *)(a1 + 32) == 0 )
    __debugbreak();
  return (*(_BYTE *)a1 & 2) == 2;
}

