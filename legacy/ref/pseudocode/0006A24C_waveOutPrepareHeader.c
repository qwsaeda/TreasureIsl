/* ea:   0x0006A24C */
/* name: waveOutPrepareHeader */
/* size: 8 */
// attributes: thunk
MMRESULT __stdcall waveOutPrepareHeader(HWAVEOUT hwo, LPWAVEHDR pwh, UINT cbwh)
{
  return __imp_waveOutPrepareHeader(hwo, pwh, cbwh);
}

