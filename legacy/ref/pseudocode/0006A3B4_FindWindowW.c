/* ea:   0x0006A3B4 */
/* name: FindWindowW */
/* size: 8 */
// attributes: thunk
HWND __stdcall FindWindowW(LPCWSTR lpClassName, LPCWSTR lpWindowName)
{
  return __imp_FindWindowW(lpClassName, lpWindowName);
}

