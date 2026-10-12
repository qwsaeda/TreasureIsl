/* ea:   0x00058664 */
/* name: SetWindowLongW */
/* size: 8 */
// attributes: thunk
LONG __stdcall SetWindowLongW(HWND hWnd, int nIndex, LONG dwNewLong)
{
  return __imp_SetWindowLongW(hWnd, nIndex, dwNewLong);
}

