/* ea:   0x00058688 */
/* name: CallWindowProcW */
/* size: 8 */
// attributes: thunk
LRESULT __stdcall CallWindowProcW(WNDPROC lpPrevWndFunc, HWND hWnd, UINT Msg, WPARAM wParam, LPARAM lParam)
{
  return __imp_CallWindowProcW(lpPrevWndFunc, hWnd, Msg, wParam, lParam);
}

