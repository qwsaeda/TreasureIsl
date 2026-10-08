/* ea:   0x0006A420 */
/* name: GetClientRect */
/* size: 8 */
// attributes: thunk
BOOL __stdcall GetClientRect(HWND hwnd, LPRECT prc)
{
  return __imp_GetClientRect(hwnd, prc);
}

