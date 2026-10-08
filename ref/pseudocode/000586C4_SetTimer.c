/* ea:   0x000586C4 */
/* name: SetTimer */
/* size: 8 */
// attributes: thunk
UINT __stdcall SetTimer(HWND hwnd, UINT idTimer, UINT uTimeOut, TIMERPROC pfnTimerProc)
{
  return __imp_SetTimer(hwnd, idTimer, uTimeOut, pfnTimerProc);
}

