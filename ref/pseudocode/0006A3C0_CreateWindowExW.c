/* ea:   0x0006A3C0 */
/* name: CreateWindowExW */
/* size: 8 */
// attributes: thunk
HWND __stdcall CreateWindowExW(
        DWORD dwExStyle,
        LPCWSTR lpClassName,
        LPCWSTR lpWindowName,
        DWORD dwStyle,
        int X,
        int Y,
        int nWidth,
        int nHeight,
        HWND hWndParent,
        HMENU hMenu,
        HINSTANCE hInstance,
        LPVOID lpParam)
{
  return __imp_CreateWindowExW(
           dwExStyle,
           lpClassName,
           lpWindowName,
           dwStyle,
           X,
           Y,
           nWidth,
           nHeight,
           hWndParent,
           hMenu,
           hInstance,
           lpParam);
}

