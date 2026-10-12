/* ea:   0x0006A228 */
/* name: ResumeThread */
/* size: 8 */
// attributes: thunk
DWORD __stdcall ResumeThread(HANDLE hThread)
{
  return __imp_ResumeThread(hThread);
}

