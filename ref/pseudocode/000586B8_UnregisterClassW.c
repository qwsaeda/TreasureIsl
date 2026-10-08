/* ea:   0x000586B8 */
/* name: UnregisterClassW */
/* size: 8 */
// attributes: thunk
BOOL __stdcall UnregisterClassW(LPCWSTR lpClassName, HINSTANCE hInstance)
{
  return __imp_UnregisterClassW(lpClassName, hInstance);
}

