/* ea:   0x0006A2E8 */
/* name: PeekMessageW */
/* size: 8 */
// attributes: thunk
BOOL __stdcall PeekMessageW(PMSG pMsg, HWND hWnd, UINT wMsgFilterMin, UINT wMsgFilterMax, UINT wRemoveMsg)
{
  return __imp_PeekMessageW(pMsg, hWnd, wMsgFilterMin, wMsgFilterMax, wRemoveMsg);
}

