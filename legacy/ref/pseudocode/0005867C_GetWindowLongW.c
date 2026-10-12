/* ea:   0x0005867C */
/* name: GetWindowLongW */
/* size: 8 */
// attributes: thunk
LONG __stdcall GetWindowLongW(HWND hWnd, int nIndex)
{
  return __imp_GetWindowLongW(hWnd, nIndex);
}

