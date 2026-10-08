/* ea:   0x0006A390 */
/* name: GetMessageW */
/* size: 8 */
// attributes: thunk
BOOL __stdcall GetMessageW(LPMSG lpMsg, HWND hWnd, UINT wMsgFilterMin, UINT wMsgFilterMax)
{
  return __imp_GetMessageW(lpMsg, hWnd, wMsgFilterMin, wMsgFilterMax);
}

