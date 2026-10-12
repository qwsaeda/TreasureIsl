/* ea:   0x0006A264 */
/* name: waveOutOpen */
/* size: 8 */
// attributes: thunk
MMRESULT __stdcall waveOutOpen(
        LPHWAVEOUT phwo,
        UINT uDeviceID,
        LPCWAVEFORMATEX pwfx,
        DWORD dwCallback,
        DWORD dwInstance,
        DWORD fdwOpen)
{
  return __imp_waveOutOpen(phwo, uDeviceID, pwfx, dwCallback, dwInstance, fdwOpen);
}

