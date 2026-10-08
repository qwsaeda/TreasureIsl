/* ea:   0x00058580 */
/* name: MessageBoxW */
/* size: 8 */
// attributes: thunk
int __stdcall MessageBoxW(HWND hWnd, LPCWSTR lpText, LPCWSTR lpCaption, UINT uType)
{
  return __imp_MessageBoxW(hWnd, lpText, lpCaption, uType);
}

