/* ea:   0x0006A240 */
/* name: waveOutWrite */
/* size: 8 */
// attributes: thunk
MMRESULT __stdcall waveOutWrite(HWAVEOUT hwo, LPWAVEHDR pwh, UINT cbwh)
{
  return __imp_waveOutWrite(hwo, pwh, cbwh);
}

