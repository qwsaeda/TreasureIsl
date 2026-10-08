/* ea:   0x0006A360 */
/* name: MoveWindow */
/* size: 8 */
// attributes: thunk
BOOL __stdcall MoveWindow(HWND hWnd, int X, int Y, int nWidth, int nHeight, BOOL bRepaint)
{
  return __imp_MoveWindow(hWnd, X, Y, nWidth, nHeight, bRepaint);
}

