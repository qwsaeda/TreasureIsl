/* ea:   0x00062D68 */
/* name: sub_62D68 */
/* size: 188 */
HWND __fastcall sub_62D68(
        HMENU dwStyle,
        HWND dwExStyle,
        int *a3,
        const WCHAR *a4,
        DWORD dwStylea,
        DWORD dwExStylea,
        HMENU a7,
        unsigned __int16 a8,
        void *lpParam)
{
  HMENU hMenu; // r6

  if ( a8 == 0 )
    return nullptr;
  sub_62E24(&dword_811A8, dwStyle + 2, dwStyle);
  hMenu = a7;
  if ( a7 == nullptr && (dwStylea & 0x40000000) != 0 )
    hMenu = dwStyle;
  return CreateWindowExW(
           dwExStylea,
           (LPCWSTR)a8,
           a4,
           dwStylea,
           *a3,
           a3[1],
           a3[2] - *a3,
           a3[3] - a3[1],
           dwExStyle,
           hMenu,
           (HINSTANCE)dword_811AC,
           lpParam);
}

