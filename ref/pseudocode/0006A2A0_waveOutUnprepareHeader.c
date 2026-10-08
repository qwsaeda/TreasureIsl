/* ea:   0x0006A2A0 */
/* name: waveOutUnprepareHeader */
/* size: 8 */
// attributes: thunk
MMRESULT __stdcall waveOutUnprepareHeader(HWAVEOUT hwo, LPWAVEHDR pwh, UINT cbwh)
{
  return __imp_waveOutUnprepareHeader(hwo, pwh, cbwh);
}

