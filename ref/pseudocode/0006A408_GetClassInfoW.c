/* ea:   0x0006A408 */
/* name: GetClassInfoW */
/* size: 8 */
// attributes: thunk
BOOL __stdcall GetClassInfoW(HINSTANCE hInstance, LPCWSTR lpClassName, LPWNDCLASSW lpWndClass)
{
  return __imp_GetClassInfoW(hInstance, lpClassName, lpWndClass);
}

