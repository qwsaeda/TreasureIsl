/* ea:   0x00062C24 */
/* name: sub_62C24 */
/* size: 48 */
int __fastcall sub_62C24(int a1)
{
  HWND WindowW; // r0

  WindowW = FindWindowW(*(LPCWSTR *)(a1 + 16), nullptr);
  if ( WindowW == nullptr )
    return 0;
  SetForegroundWindow((HWND)((unsigned int)WindowW | 1));
  return 1;
}

