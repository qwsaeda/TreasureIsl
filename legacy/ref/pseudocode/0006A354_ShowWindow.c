/* ea:   0x0006A354 */
/* name: ShowWindow */
/* size: 8 */
// attributes: thunk
BOOL __stdcall ShowWindow(HWND hwnd, INT nCmdShow)
{
  return __imp_ShowWindow(hwnd, nCmdShow);
}

