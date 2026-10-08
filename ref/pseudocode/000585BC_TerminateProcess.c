/* ea:   0x000585BC */
/* name: TerminateProcess */
/* size: 8 */
// attributes: thunk
BOOL __stdcall TerminateProcess(HANDLE hProcess, DWORD uExitCode)
{
  return __imp_TerminateProcess(hProcess, uExitCode);
}

